# Web Server Using Rust

## Principals:
1. To realise a web server, we should implement multi-threads to handle multiple requests simultaneously without blocking the main thread
2. In order to do that we have to create a **thread pool**, in which a number of threads are spawned, and are listening and ready to handle tasks.
3. When a request is received, it will be saved on a **queue**, and we will look for the nearest free thread, and assign it to that task.
4. When we find a free thread, we pop the last request and assign it to that thread
5. When they are all busy, we wait till one of the threads is free, then we assign the last request to it (pop)
6. We should specify the number threads of the pool to avoid the **Denial of Service (Dos) attack**
7. We should also create a **worker**, that will have a task to perform, and an id, because a thread should have a known closure at its creation, that's why a worker generally will be passed to it.