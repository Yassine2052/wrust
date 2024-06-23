use std::cmp::max;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

// This is what we will send through the channel to handle the request
type Job = Box<dyn FnOnce() + Send + 'static>;

// The worker will have an id and JoinHandle to drop the thread when finished
struct Worker{
    id: usize,
    thread: Option<JoinHandle<()>>,
}

// This is the group of threads we launched, and the channel sender
pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: Sender<Job>
}

impl Worker{
    // Here we pass an id, and the channel receiver cloned by the Arc, and contained in the Mutex
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = Some(thread::spawn(move || loop {
            // Lock the mutex so one thread handles a job at once
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker Id {id} Got Job. Executing...");

            job();
        }));

        Worker{
            id,
            thread
        }
    }
}

impl ThreadPool{
    /// Create a new ThreadPool.
    /// The size is the number of threads in the pool.
    /// # Panics
    /// The `new` function will panic if the size is zero.
    pub fn new(mut size: usize) -> ThreadPool{
        size = max(size, 1);

        // Create a channel
        let (sender, receiver) = channel();

        // Create empty workers vec
        let mut workers = Vec::with_capacity(size);

        // To pass the receiver to multiple owners we need to use the Arc, and to prevent race access we use the mutex to lock
        let receiver = Arc::new(Mutex::new(receiver));

        // Create the given number of workers
        for id in 0..size{
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
        where
        // Closure should be run only once, and Implements the Send trait, also it should have a 'static lifetime because we do not know how much time the instance will last
        F: FnOnce() + Send + 'static,
    {
        // Create a job with the closure
        let job = Box::new(f);

        // Send the job to the available thread (availability depends on OS Scheduler)
        self.sender.send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}