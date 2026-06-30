use std::{sync::{Arc, Condvar, Mutex}, thread::{self, JoinHandle}, time::{Duration, Instant}};


/*
    La struct DelayedExecutor permette di eseguire funzioni in modo asincrono, dopo un certo intervallo di tempo. 
    Essa offre tre metodi:

    new() -> Self crea un nuovo DelayedExecutor

    execute<F: FnOnce()+Send+'static>(f:F, delay: Duration) -> bool
        se il DelayedExecutor è aperto, accoda la funzione f che dovrà essere eseguita non prima che sia
        trascorso un intervallo pari a delay e restituisce true; se invece il DelayedExecutor è chiuso, restituisce false.

    close(drop_pending_tasks: bool) chiude il DelayedExecutor;
        se drop_pending_tasks è true, le funzioni in attesa di essere eseguite vengono eliminate, altrimenti vengono eseguite a tempo debito.

    DelayedExecutor è thread-safe e può essere utilizzato da più thread contemporaneamente. 
    I task sottomessi al DelayedExecutor devono essere eseguiti in ordine di scadenza. 
    All'atto della distruzione di un DelayedExecutor, tutti i task in attesa sono eliminati, 
    ma se è in corso un'esecuzione questa viene portata a termine evitando di creare corse critiche. 
    Si implementi questa struct in linguaggio Rust.


*/


// =========== Inserisci Implementazione 


// =========== Test


fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_execute_returns_true_when_open() {
        let executor = DelayedExecutor::new();
        let executed = Arc::new(Mutex::new(false));

        let executed_clone = executed.clone();
        let res = executor.execute(
            move || {
                *executed_clone.lock().unwrap() = true;
            },
            Duration::from_millis(50),
        );

        assert!(res, "Expected execute() to return true when executor is open");

        thread::sleep(Duration::from_millis(100));
        assert!(*executed.lock().unwrap(), "Task should have been executed");
    }

    #[test]
    fn test_execute_returns_false_when_closed() {
        let executor = DelayedExecutor::new();
        executor.close(true);

        let res = executor.execute(|| println!("Should not run"), Duration::from_millis(10));

        assert!(!res, "Expected execute() to return false when executor is closed");
    }

    #[test]
    fn test_close_drops_pending_tasks() {
        let executor = DelayedExecutor::new();
        let executed = Arc::new(Mutex::new(false));

        let executed_clone = executed.clone();
        let _ = executor.execute(
            move || {
                *executed_clone.lock().unwrap() = true;
            },
            Duration::from_millis(200),
        );

        executor.close(true); // chiusura eliminando i task pendenti
        thread::sleep(Duration::from_millis(300));

        assert!(
            !*executed.lock().unwrap(),
            "Task should not have been executed because it was dropped"
        );
    }

    #[test]
    fn test_close_runs_pending_tasks() {
        let executor = DelayedExecutor::new();
        let executed = Arc::new(Mutex::new(false));

        let executed_clone = executed.clone();
        let _ = executor.execute(
            move || {
                *executed_clone.lock().unwrap() = true;
            },
            Duration::from_millis(100),
        );

        executor.close(false); // chiusura mantenendo i task
        thread::sleep(Duration::from_millis(200));

        assert!(
            *executed.lock().unwrap(),
            "Task should have been executed because it was preserved"
        );
    }

    #[test]
    fn test_tasks_run_in_order_of_deadline() {
        let executor = DelayedExecutor::new();
        let results = Arc::new(Mutex::new(Vec::new()));

        let r1 = results.clone();
        executor.execute(
            move || r1.lock().unwrap().push(1),
            Duration::from_millis(200),
        );

        let r2 = results.clone();
        executor.execute(
            move || r2.lock().unwrap().push(2),
            Duration::from_millis(100),
        );

        thread::sleep(Duration::from_millis(300));

        let results = results.lock().unwrap().clone();
        assert_eq!(results, vec![2, 1], "Tasks should execute in order of expiration");
    }

    #[test]
    fn test_executor_drops_tasks_on_drop() {
        let executed = Arc::new(Mutex::new(false));
        {
            let executor = DelayedExecutor::new();
            let executed_clone = executed.clone();
            executor.execute(
                move || {
                    *executed_clone.lock().unwrap() = true;
                },
                Duration::from_millis(500),
            );
            // executor viene droppato qui
        }

        thread::sleep(Duration::from_millis(600));
        assert!(
            !*executed.lock().unwrap(),
            "Task should not run after executor was dropped"
        );
    }

    #[test]
    fn test_concurrent_submissions() {
        let executor = Arc::new(DelayedExecutor::new());
        let counter = Arc::new(Mutex::new(0));

        let mut handles = vec![];
        for _ in 0..10 {
            let ex = executor.clone();
            let c = counter.clone();
            handles.push(thread::spawn(move || {
                ex.execute(
                    move || {
                        *c.lock().unwrap() += 1;
                    },
                    Duration::from_millis(50),
                );
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        thread::sleep(Duration::from_millis(100));
        assert_eq!(
            *counter.lock().unwrap(),
            10,
            "All submitted tasks should have executed"
        );
    }
}