# MUD Server
A concept study for ANSI capable mud server to interoperate with the BalcCon 
Badge.

## Software Design

### High Level Architecture
TODO - connection manager / world task

### Concurrency Model
In order to allow multiple users access the mud we need some sort of
concurrency. The main models to choose from are:

- OS threads

    Easy to use, however has a big overhead and needs us to deal with 
    synchronization problems. 

- Coroutines

    Not well supported and not ideal for server code.

- Event-driven programming

    Usually results in non linear control flow. Data flow and errors are hard
    to follow.

- Actor model

    Divides concurent calculations in actors. Actors communicate through
    fallible messages. Unclear how to address flow control and retry logic.

- Rust Async

    Performant for high amount of parallel operations with high latency
    (especially I/O operations). Not ideal for parallell computations.

(***DECISION***) The async model will be used. The software will be implented
    in Rust using tokio as the engine for async.


## Asorted notes:

Protocol:
- General considerations message format: Type | 
- Command message format: CMD
- Data message format:

AI Actors:
- Run in own thread each like a simulated player



Misc stuff:
- Client keypair: Must be ed25519
- Calling from a client: ssh -i ~/.ssh/id_ed25519   -o "UserKnownHostsFile=/dev/null" -o PreferredAuthentications=publickey -o StrictHostKeyChecking=no localhost -p 2222

## Grammar
Support for the following clauses:

0. (sbuject implied as self) verb
    Examples:
        - Look

1. (subject implied as self) verb - object.
    Examples:
        - Look at object
        - Read book
        - Open port

1. (subject implied as self) verb - adjective - object
    Examples:
        - Look at old book
        - Read yellow paper
        - Open rusty port

1. verb - object - preposition - object
    Examples:
        - Attack ICE with exploit
        - Examine port with portscan

1. verb - adjective - object - preposition- adjective - object
        - Attack black ICE with weak exploit
        - Use ancient glasses to read old book

In order to develop a language that the programm understands, we need to develop
a syntax and semantics. 


We provide the syntay in BNF as a basis to build the parser. The BNF can be
tested at [BNF Playgound](https://bnfplayground.pauliankline.com/?bnf=%3Csentence%3E%20%3A%3A%3D%20%3Caction%3E%20%7C%20%3Ccommand%3E%0A%3Caction%3E%20%3A%3A%3D%20%3Cverb%3E%20%3Cblank%3E%20%3Cadverblist%3E%20%3Cobject%3E%20(%22.%22%20%7C%20E)%0A%3Ccommand%3E%20%3A%3A%3D%20%22help%22%20(%3Cblank%3E%20%3Ctopic%3E%20%7C%20E)%20%7C%20%22inventory%22%0A%3Cadverblist%3E%20%3A%3A%3D%20%3Cadverb%3E%20%3Cblank%3E%20%3Cadverblist%3E%20%7C%20E%0A%3Cadverb%3E%20%3A%3A%3D%20%22quickly%22%20%7C%20%22slowly%22%0A%3Cdo%3E%20%3A%3A%3D%20%22do%22%0A%3Cverb%3E%20%3A%3A%3D%20%22look%22%20%7C%20%22read%22%20%7C%20%22enter%22%20%7C%20%22connect%22%20%7C%20%22access%22%20%7C%20%22open%22%0A%2F*%20Need%20to%20replace%20with%20dynamic%20content%20from%20game%20*%2F%0A%3Cobject%3E%20%3A%3A%3D%20%3Carticle%3E%20(%22port%22%20%7C%20%22ram%20bank%22%20%7C%20%22quickhack%22)%0A%3Carticle%3E%20%3A%3A%3D%20(%22the%22%20%3Cblank%3E%20%7C%20E)%0A%2F*%20Need%20to%20replace%20with%20dynamic%20content%20from%20game%20*%2F%0A%3Ctopic%3E%20%3A%3A%3D%20%22verbs%22%20%7C%20%22inventory%22%20%7C%20%22combat%22%20%0A%3Cblank%3E%20%3A%3A%3D%20%22%20%22%2B&name=)


// TODO - maybe just allow one adverb for simplicity sake
```
 <sentence> ::= <action> | <command>
 <action> ::= <verb> <blank> <adverblist> <blank> <object> ("." | E)
 <command> ::= "help" (<blank> <topic> | E) | "inventory"
 <adverblist> ::= <adverb> | <adverb> (","+ <blank>* | <blank>+) <adverblist> | E
 <adverb> ::= "quickly" | "slowly"
 <do> ::= "do"
 <verb> ::= "look" | "read" | "enter" | "connect" | "access" | "open"
 <object> ::= <article> ("port" | "ram bank" | "quickhack")
 <article> ::= ("the" <blank> | E)
 <topic> ::= "verbs" | "inventory" | "combat" 
 <blank> ::= " "+
```

Terminal cursor movement Test\r\n\x1B[0;0HThis is a test"
See also: https://docs.rs/termion/latest
See also: https://www.lihaoyi.com/post/BuildyourownCommandLinewithANSIescapecodes.html