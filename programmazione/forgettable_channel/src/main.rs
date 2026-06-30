
/*
Implementare la funzione forgettable_channel<T>() che crea un canale multi-produttore/ricevitore-singolo le
cui semantiche estendono quelle di std::sync::mpsc::channel<T> con la possibilità di annullare un messaggio
già accodato prima che venga elaborato dal ricevitore.

I tratti che seguono definiscono il contratto pubblico del canale; non devono essere modificati.
Forgettable Handle restituito da send(). 

Il metodo forget() tenta di annullare il messaggio corrispondente ancora in coda:
    -   restituisce true se il messaggio era ancora in attesa o era già stato annullato;
    -   restituisce false se il ricevitore lo aveva già elaborato o se il canale è stato chiuso prima che il
        messaggio fosse accodato


ForgettableSender<T>: Clone Lato mittente del canale.
    send(t) restituisce Some(handle) se il messaggio è stato accodato con successo, None se il ricevitore non
    esiste più.


ForgettableReceiver<T> Lato ricevitore del canale.
    recv() blocca finché non è disponibile un messaggio non annullato e restituisce None solo quando il
    canale è chiuso e la coda è vuota. I messaggi annullati vengono scartati silenziosamente


Suggerimenti implementativi
1. Creare la coppia mittente/ricevitore sottostante (ad esempio tramite std::sync::mpsc::channel<T>).
2. Avvolgere il mittente in un tipo concreto che implementi sia ForgettableSender<T> sia Clone, così da
supportare più produttori.
3. Avvolgere il ricevitore in un tipo concreto che implementi ForgettableReceiver<T>.

*/

/// Rappresenta un riferimento a un messaggio in volo che può essere annullato
/// prima che il ricevitore lo elabori.
///
/// Un valore che implementa `Forgettable` viene restituito da
/// [`ForgettableSender::send`] e consente al mittente di "dimenticare"
/// il messaggio già accodato.
pub trait Forgettable {
/// Tenta di annullare il messaggio associato.
///
/// Restituisce `true` se il messaggio era ancora in attesa di essere
/// ricevuto (oppure era già stato annullato da una chiamata precedente),
/// `false` se il ricevitore lo aveva già elaborato o se il canale è
/// stato chiuso.
///
/// La chiamata è idempotente: invocare `forget` più volte sullo stesso
/// handle mentre il messaggio è ancora in attesa restituisce sempre `true`.
fn forget(&self) -> bool;
}
/// Rappresenta il lato mittente di un canale di messaggi dimenticabili.
///
/// Implementa [`Clone`] per consentire a più thread di condividere lo stesso
/// endpoint di invio (canale multi-produttore, ricevitore singolo).
pub trait ForgettableSender<T:Send+'static>: Clone {
/// Invia il valore `t` nel canale.
///
/// Restituisce `Some(handle)` in caso di successo: `handle` implementa
/// [`Forgettable`] e può essere usato in seguito per annullare il
/// messaggio prima che venga ricevuto.
/// Restituisce `None` se il ricevitore è stato già eliminato (canale
/// disconnesso).
fn send(&self, t:T) -> Option<impl Forgettable+'static>;
}
/// Rappresenta il lato ricevitore di un canale di messaggi dimenticabili.
///
/// Riceve i messaggi in ordine FIFO, saltando silenziosamente quelli che
/// sono stati annullati tramite [`Forgettable::forget`] prima di essere
/// estratti dalla coda.
pub trait ForgettableReceiver<T:Send+'static> {
/// Blocca il thread corrente finché non è disponibile un messaggio
/// non annullato, quindi lo restituisce.
///
/// Restituisce `None` quando tutti i mittenti sono stati eliminati e
/// la coda è vuota (canale chiuso).
/// I messaggi annullati vengono consumati dalla coda e scartati
/// internamente; il chiamante non li vede mai.
fn recv(&self) -> Option<T>;
}

/// Crea un nuovo canale dimenticabile e restituisce la coppia
/// `(`[`ForgettableSender`]`, `[`ForgettableReceiver`]`)`.
///
/// Il canale è illimitato e può trasportare qualsiasi tipo `T: Send + 'static`.
///
/// # Da implementare
///
/// Fornire un'implementazione concreta che:
/// 1. Crei la coppia mittente/ricevitore sottostante.
/// 2. Avvolga il mittente in un tipo che implementa [`ForgettableSender`].
/// 3. Avvolga il ricevitore in un tipo che implementa [`ForgettableReceiver`].
pub fn forgettable_channel<T: Send + 'static>() -> (impl ForgettableSender<T>, impl ForgettableReceiver<T>) {
todo!()
}


fn main() {
    println!("Hello, world!");
}
