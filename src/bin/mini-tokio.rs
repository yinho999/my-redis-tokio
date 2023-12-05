use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, mpsc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};
use futures::task::ArcWake;
use tokio::sync::Notify;
use std::cell::RefCell;

struct Delay {
    when: Instant,
    // This is `Some` when we have spawned a thread, `None` when we haven't.
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
            -> Poll<()>
    {
        // First, if this is the first time the future is called, spawn the
        // timer thread. If the timer thread is already running, ensure the
        // stored `Waker` matches the current task's waker.
        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();

            // Check if the stored waker matches the current task's waker.
            // This is necessary as the `Delay` future instance may move to
            // a different task between calls to `poll`. If this happens, the
            // waker contained by the given `Context` will differ and we
            // must update our stored waker to reflect this change.
            if !waker.will_wake(cx.waker()) {
                *waker = cx.waker().clone();
            }
        } else {
            // Get when value
            let when = self.when;
            // Create a handler to the waker for the current task
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());
            // This is the first time `poll` is called, Spawn a timer thread
            thread::spawn(move || {
                let now = Instant::now();
                if now < when {
                    thread::sleep(when - now);
                }
                // the duration has elapsed, notify the task
                let waker = waker.lock().unwrap();
                // Notifying the calling task - can use wake_by_ref
                // notify the calling task and the executor can ensure the task is scheduled again.
                waker.wake_by_ref();
            });
        }
        // Once the waker is stored and the timer thread is started, it is
        // time to check if the delay has completed. This is done by
        // checking the current instant. If the duration has elapsed, then
        // the future has completed and `Poll::Ready` is returned.
        if Instant::now() >= self.when {
            Poll::Ready(())
        } else {
            // The duration has not elapsed, the future has not completed so
            // return `Poll::Pending`.
            //
            // The `Future` trait contract requires that when `Pending` is
            // returned, the future ensures that the given waker is signalled
            // once the future should be polled again. In our case, by
            // returning `Pending` here, we are promising that we will
            // invoke the given waker included in the `Context` argument
            // once the requested duration has elapsed. We ensure this by
            // spawning the timer thread above.
            //
            // If we forget to invoke the waker, the task will hang
            // indefinitely.
            Poll::Pending
        }
    }
}


struct Task {
    // The `Mutex` is to make `Task` implement `Sync`. Only
    // one thread accesses `future` at any given time. The
    // `Mutex` is not required for correctness. Real Tokio
    // does not use a mutex here, but real Tokio has
    // more lines of code than can fit in a single tutorial
    // page.
    future: Mutex<Pin<Box<dyn Future<Output=()> + Send>>>,
    // The `Sender` is used to send the task itself to the executor.
    executor: mpsc::Sender<Arc<Task>>,
}

impl Task {
    fn poll(self: Arc<Self>) {
        // Create a waker from the `Task` instance. this uses the `ArcWake` trait impl.
        let waker = futures::task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        // No other thread should access the future while we are polling it.
        let mut future = self.future.lock().unwrap();
        // Poll the future
        let _ = future.as_mut().poll(&mut cx);
    }
    /// Spawns a new task with the given future.
    ///
    /// Initialize a new Task harness with the given future and send it onto `sender`. The receiver half of the channel will get the task and execute it.
    fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>)
        where
            F: Future<Output=()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });
        // Send the task to the executor
        sender.send(task).unwrap();
    }

    // `schedule` takes send itself to the executor and schedule for execution
    fn schedule(self: &Arc<Self>) {
        // Send the task to the executor
        self.executor.send(self.clone()).unwrap();
    }
}

impl ArcWake for Task {
    // `wake_by_ref` is called by the executor when the task should be awoken.
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}


struct MiniTokio {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl MiniTokio {
    /// Initialize a new `MiniTokio` instance.
    fn new() -> Self {
        let (sender, scheduled) = mpsc::channel();
        MiniTokio { scheduled, sender }
    }

    /// Spawn a future onto the mini-tokio instance.
    ///
    /// The given future is wrapped into a `Task` harness and pushed into the scheduled queue.
    /// The future is then scheduled for execution.
    fn spawn<F>(&mut self, future: F)
        where
            F: Future<Output=()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}
async fn delay(dur: Duration) {
    let when = Instant::now() + dur;
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    thread::spawn(move || {
        let now = Instant::now();

        if now < when {
            thread::sleep(when - now);
        }

        notify_clone.notify_one();
    });


    notify.notified().await;
}

// Used to track the current mini-tokio instance so that the `spawn` function is
// able to schedule spawned tasks.
thread_local! {
    static CURRENT: RefCell<Option<mpsc::Sender<Arc<Task>>>> =
        RefCell::new(None);
}

// An equivalent to `tokio::spawn`. When entering the mini-tokio executor, the
// `CURRENT` thread-local is set to point to that executor's channel's Send
// half. Then, spawning requires creating the `Task` harness for the given
// `future` and pushing it into the scheduled queue.
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    CURRENT.with(|cell| {
        let borrow = cell.borrow();
        let sender = borrow.as_ref().unwrap();
        Task::spawn(future, sender);
    });
}

fn main() {

    // Create the mini-tokio instance.
    let mut mini_tokio = MiniTokio::new();

    // Spawn the root task. All other tasks are spawned from the context of this
    // root task. No work happens until `mini_tokio.run()` is called.
    mini_tokio.spawn(async {
        // Spawn a task
        tokio::spawn(async {
            // Wait for a little bit of time so that "world" is printed after
            // "hello"
            delay(Duration::from_millis(100)).await;
            println!("world");
        });

        // Spawn a second task
        tokio::spawn(async {
            println!("hello");
        });

        // We haven't implemented executor shutdown, so force the process to exit.
        delay(Duration::from_millis(200)).await;
        std::process::exit(0);
    });

    // Start the mini-tokio executor loop. Scheduled tasks are received and
    // executed.
    mini_tokio.run();
}