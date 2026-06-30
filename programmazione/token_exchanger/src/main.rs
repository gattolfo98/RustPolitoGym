use std::{
    sync::{Arc, Condvar, Mutex},
    time::Instant,
};


/*
Un applicativo software multithread fa accesso ai servizi di un server remoto, attraverso richieste di tipo HTTP.
Tali richieste devono includere un token di sicurezza che identifica l'applicativo stesso e ne autorizza l'accesso.
Per motivi di sicurezza, il token ha una validità limitata nel tempo (qualche minuto) e deve essere rinnovato alla sua scadenza.
Il token viene ottenuto attraverso una funzione (fornita esternamente e conforme al tipo **TokenAcquirer**) che restituisce alternativamente un token e la sua data di scadenza o un messaggio di errore se non è possibile fornirlo.
Poiché la emissione richiede un tempo apprezzabile (da alcune centinaia di millisecondi ad alcuni secondi), si vuole centralizzare la gestione del token,
per evitare che più thread ne facciano richiesta in contemporanea.

A tale scopo deve essere implementata la struct TokenManager che si occupa di gestire il rilascio, il rinnovo e la messa a disposizione del token a chi ne abbia bisogno, secondo la logica di seguito indicata.

La struct **TokenManager** offre i seguenti metodi:

```Rust
type TokenAcquirer = dyn Fn() -> Result<(String, Instant), String> + Sync;

pub fn new(acquire_token: Box<TokenAcquirer> ) -> Self
pub fn get_token(&self) -> Result<String, String>
pub fn try_get_token(&self) -> Option<String>
```

Al proprio interno, la struct TokenManager mantiene 3 possibili stati:

- Empty - indica che non è ancora stato richiesto alcun token;
- Pending - indica che è in corso una richiesta di acquisizione del token;
- Valid - indica che è disponibile un token in corso di validità;

Il metodo `new(...)` riceve il puntatore alla funzione in grado di acquisire il token. Essa opera in modalità pigra e si limita a creare un'istanza della struttura con le necessarie informazioni per gestire il suo successivo comportamento.

Il metodo `get_token(...)` implementa il seguente comportamento:

- Se lo stato è Empty, passa allo stato Pending e invoca la funzione per acquisire il token; se questa ritorna un risultato valido, memorizza il token e la sua scadenza, porta lo stato a Valid e restituisce copia del token stesso; <br>se, invece, questa restituisce un errore, pone lo stato a Empty e restituisce l'errore ricevuto.
- Se lo stato è Pending, attende senza consumare cicli di CPU che questo passi ad un altro valore, dopodiché si comporta di conseguenza.
- Se lo stato è Valid e il token non risulta ancora scaduto, ne restituisce una copia; altrimenti pone lo stato ad Pending e inizia una richiesta di acquisizione, come indicato sopra.

Il metodo `try_get_token(...)` implementa il seguente comportamento:

- Se lo stato è Valid e il token non è scaduto, restituisce una copia del token opportunamente incapsulata in un oggetto di tipo Option. In tutti gli altri casi restituisce None.

Si implementi tale struttura nel linguaggio Rust.

A supporto della validazione del codice realizzato si considerino i seguenti test (due dei quali sono forniti con la relativa implementazione, i restanti sono solo indicati e possono essere opportunamente completati):

```Rust
    #[test]
    fn a_new_manager_contains_no_token() {
        let a: Box<TokenAcquirer> = Box::new(|| Err("failure".to_string()));
        let manager = TokenManager::new(a);
        assert!(manager.try_get_token().is_none());
    }
    #[test]
    fn a_failing_acquirer_always_returns_an_error() {
        let a: Box<TokenAcquirer> = Box::new(|| Err("failure".to_string()));
        let manager = TokenManager::new(a);
        assert_eq!(manager.get_token(), Err("failure".to_string()));
        assert_eq!(manager.get_token(), Err("failure".to_string()));
    }
    #[test]
    fn a_successful_acquirer_always_returns_success() {
      //...to be implemented
    }
    #[test]
    fn a_slow_acquirer_causes_other_threads_to_wait() {
      //...to be implemented
    }
```

*/



// ======== Inserisci implementazione: 



// ======== TEST

fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::thread;
    use std::time::{Duration, Instant};

    // ── Già forniti ──────────────────────────────────────────────────────────

    #[test]
    fn a_new_manager_contains_no_token() {
        let a: Box<TokenAcquirer> = Box::new(|| Err("failure".to_string()));
        let manager = TokenManager::new(a);
        assert!(manager.try_get_token().is_none());
    }

    #[test]
    fn a_failing_acquirer_always_returns_an_error() {
        let a: Box<TokenAcquirer> = Box::new(|| Err("failure".to_string()));
        let manager = TokenManager::new(a);
        assert_eq!(manager.get_token(), Err("failure".to_string()));
        assert_eq!(manager.get_token(), Err("failure".to_string()));
    }

    // ── Da implementare ──────────────────────────────────────────────────────

    #[test]
    fn a_successful_acquirer_always_returns_success() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_c = Arc::clone(&call_count);

        let a: Box<TokenAcquirer> = Box::new(move || {
            call_count_c.fetch_add(1, Ordering::SeqCst);
            Ok(("valid-token".to_string(), Instant::now() + Duration::from_secs(60)))
        });

        let manager = TokenManager::new(a);

        // get_token deve restituire il token
        assert_eq!(manager.get_token(), Ok("valid-token".to_string()));

        // try_get_token deve restituirlo incapsulato (il token è ancora valido)
        assert_eq!(manager.try_get_token(), Some("valid-token".to_string()));

        // Una seconda chiamata deve restituire lo stesso token SENZA richiamare l'acquirer
        assert_eq!(manager.get_token(), Ok("valid-token".to_string()));
        assert_eq!(
            call_count.load(Ordering::SeqCst), 1,
            "L'acquirer deve essere chiamato una sola volta se il token è ancora valido"
        );
    }

    #[test]
    fn a_slow_acquirer_causes_other_threads_to_wait() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_c = Arc::clone(&call_count);
        let acquirer_delay = Duration::from_millis(400);

        let a: Box<TokenAcquirer> = Box::new(move || {
            call_count_c.fetch_add(1, Ordering::SeqCst);
            thread::sleep(acquirer_delay);
            Ok(("slow-token".to_string(), Instant::now() + Duration::from_secs(60)))
        });

        let manager = Arc::new(TokenManager::new(a));
        let mut handles = vec![];

        let start = Instant::now();

        for _ in 0..6 {
            let m = Arc::clone(&manager);
            handles.push(thread::spawn(move || m.get_token()));
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let elapsed = start.elapsed();

        // Tutti i thread devono ricevere il token
        for r in &results {
            assert_eq!(r, &Ok("slow-token".to_string()));
        }

        // L'acquirer deve essere stato chiamato una sola volta
        assert_eq!(
            call_count.load(Ordering::SeqCst), 1,
            "L'acquirer non deve essere invocato in parallelo da più thread"
        );

        // Il tempo totale deve essere vicino al singolo delay, non N volte
        assert!(
            elapsed < acquirer_delay * 2,
            "I thread hanno atteso in serie invece che in parallelo: {:?}", elapsed
        );
    }

    // ── Extra ────────────────────────────────────────────────────────────────

  #[test]
fn try_get_token_returns_none_while_state_is_pending() {
    // Una Barrier a 2 garantisce che chiamiamo try_get_token
    // ESATTAMENTE quando l'acquirer è in esecuzione (stato = Pending)
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let barrier_c = Arc::clone(&barrier);

    let a: Box<TokenAcquirer> = Box::new(move || {
        // Segnala: siamo dentro l'acquirer → stato certamente Pending
        barrier_c.wait();
        // Restiamo in esecuzione abbastanza a lungo da permettere il controllo
        thread::sleep(Duration::from_millis(600));
        Ok(("token".to_string(), Instant::now() + Duration::from_secs(60)))
    });

    let manager = Arc::new(TokenManager::new(a));
    let m2 = Arc::clone(&manager);

    let handle = thread::spawn(move || m2.get_token());

    // Aspettiamo che l'acquirer abbia CERTAMENTE iniziato
    // (e quindi lo stato sia Pending)
    barrier.wait();

    let start  = Instant::now();
    let result = manager.try_get_token();
    let elapsed = start.elapsed();

    assert!(
        result.is_none(),
        "try_get_token deve tornare None durante Pending, ha restituito {:?}", result
    );
    assert!(
        elapsed < Duration::from_millis(100),
        "try_get_token non deve bloccarsi in stato Pending: {:?}", elapsed
    );

    handle.join().unwrap();
}

    #[test]
    fn token_is_renewed_after_expiry() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_c = Arc::clone(&call_count);

        let a: Box<TokenAcquirer> = Box::new(move || {
            let n = call_count_c.fetch_add(1, Ordering::SeqCst);
            Ok((
                format!("token-v{}", n),
                // Prima chiamata: scade tra 100ms; seconda: lunga validità
                Instant::now() + if n == 0 { Duration::from_millis(100) } else { Duration::from_secs(60) },
            ))
        });

        let manager = TokenManager::new(a);

        let first = manager.get_token().unwrap();
        assert_eq!(first, "token-v0");

        // Aspettiamo la scadenza
        thread::sleep(Duration::from_millis(200));

        // Dopo la scadenza, deve essere acquisito un nuovo token
        let second = manager.get_token().unwrap();
        assert_eq!(second, "token-v1");
        assert_ne!(first, second, "Il token rinnovato deve essere diverso dal precedente");
        assert_eq!(
            call_count.load(Ordering::SeqCst), 2,
            "L'acquirer deve essere chiamato una seconda volta al rinnovo"
        );
    }

    #[test]
    fn try_get_token_returns_none_after_expiry() {
        let a: Box<TokenAcquirer> = Box::new(|| {
            Ok(("short-lived".to_string(), Instant::now() + Duration::from_millis(150)))
        });

        let manager = TokenManager::new(a);

        // Acquisizione iniziale
        assert_eq!(manager.get_token(), Ok("short-lived".to_string()));

        // Ancora valido
        assert!(manager.try_get_token().is_some());

        // Aspettiamo la scadenza
        thread::sleep(Duration::from_millis(250));

        // Scaduto: try_get_token deve tornare None
        assert!(
            manager.try_get_token().is_none(),
            "try_get_token deve tornare None dopo la scadenza del token"
        );
    }

    #[test]
    fn after_failure_state_resets_to_empty_and_retries() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_c = Arc::clone(&call_count);

        let a: Box<TokenAcquirer> = Box::new(move || {
            let n = call_count_c.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                Err("errore temporaneo".to_string())
            } else {
                Ok(("recovered-token".to_string(), Instant::now() + Duration::from_secs(60)))
            }
        });

        let manager = TokenManager::new(a);

        // Prima chiamata: l'acquirer fallisce → Err + stato torna a Empty
        assert_eq!(
            manager.get_token(),
            Err("errore temporaneo".to_string())
        );

        // In stato Empty, try_get_token deve tornare None
        assert!(
            manager.try_get_token().is_none(),
            "Dopo un fallimento lo stato deve tornare Empty"
        );

        // Seconda chiamata: l'acquirer ha successo → deve riprovare
        assert_eq!(
            manager.get_token(),
            Ok("recovered-token".to_string())
        );
        assert_eq!(
            call_count.load(Ordering::SeqCst), 2,
            "L'acquirer deve essere invocato nuovamente dopo un fallimento"
        );
    }

    #[test]
    fn error_message_is_forwarded_verbatim() {
        // Il messaggio di errore deve essere propagato inalterato
        let a: Box<TokenAcquirer> =
            Box::new(|| Err("HTTP 503 – Service Unavailable".to_string()));
        let manager = TokenManager::new(a);
        assert_eq!(
            manager.get_token(),
            Err("HTTP 503 – Service Unavailable".to_string())
        );
    }
}