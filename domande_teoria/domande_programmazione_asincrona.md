# Domande sulla programmazione asincrona 

## Generate con IA

### 1. Qual è la differenza concettuale tra usare i thread e usare l'esecuzione asincrona?
### 2. Nell'event loop, quali sono i tre passi che vengono ripetuti in ciclo?
### 3. Cosa si intende per "callback hell", e perché è un problema (oltre alla semplice leggibilità)?
### 4. Nella versione con i Future `(.and_then(...))`, cosa rappresenta concretamente `and_then`? Che differenza c'è rispetto a passare una callback diretta come si faceva prima?
### 5. Cosa significa dire che `Poll::Pending` viene restituito da `poll()`? E cosa succede subito dopo, concettualmente, perché la macchina a stati possa "ripartire"?
### 6. Quando scrivi:
```rust 
async fn copy(file1: String, file2: String) -> Result<()> { ... }
```
Che tipo ha davvero il valore ritornato da `copy(...)` quando la chiami? E cosa contiene/rappresenta quel valore nel momento in cui viene creato (cioè appena prima di fare il primo .await su di esso)?


### 7. A cosa serve #[tokio::main]? Perché non si può semplicemente scrivere async fn main() { ... } senza quella macro?

### 8. Qual è la differenza fondamentale tra un Future "normale" (es. creato chiamando una async fn senza fare nulla con il risultato) e un task creato con tokio::spawn(...)? Quando inizia davvero l'esecuzione nei due casi?

### 9. Cosa restituisce task.await quando task è un JoinHandle ottenuto da tokio::spawn(...)? Perché serve .unwrap() in task.await.unwrap()?

### 10. Che differenza c'è tra tokio::join!(f1, f2) e scrivere semplicemente:
```rust
let r1 = f1.await;
let r2 = f2.await;
```

### 11. Cosa fa tokio::select! e qual è la conseguenza "pericolosa" a cui bisogna stare attenti quando lo si usa? Fammi un esempio di scenario in cui questo potrebbe causare un bug se non ci si pensa.

### 12. Piccolo scenario pratico: hai 3 richieste HTTP indipendenti da fare, e ti serve il risultato di tutte e tre prima di proseguire. Quale strumento tra quelli visti useresti? E se invece ti bastasse la risposta più veloce tra le tre (es. stai interrogando 3 server mirror equivalenti)?


### Perché nell'esempio con Arc<Mutex<i32>> non si può semplicemente condividere data tra i vari task spawnati senza Arc? Cosa impedirebbe il compilatore, e perché?

### Qual è la differenza pratica tra tokio::sync::Mutex e std::sync::Mutex? In quale dei due casi useresti l'uno o l'altro, e perché?

### Descrivi la differenza concettuale tra i quattro canali (oneshot, mpsc, broadcast, watch) in termini di numero di mittenti e numero di riceventi.

### Nel canale watch, cosa succede se il mittente chiama tx.send() tre volte di seguito molto rapidamente, prima che il receiver abbia il tempo di controllare rx.changed()? Il receiver vedrà tutti e tre i valori, o no? Confrontalo con cosa succederebbe nello stesso scenario con un canale broadcast.

### Nel canale mpsc, cosa rappresenta il numero passato a mpsc::channel(100)? Cosa succede se un task produttore chiama tx.send(...).await quando il canale è pieno?

### Scenario pratico: stai scrivendo un server che deve notificare a tutti i client connessi (potenzialmente centinaia) ogni volta che arriva un nuovo messaggio in una chat room, e ogni client deve ricevere ogni singolo messaggio, in ordine, senza perderne nessuno. Quale canale useresti tra quelli visti, e perché escluderesti gli altri tre?



