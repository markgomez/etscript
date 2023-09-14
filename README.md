# ETscript

## Why?

After spending a few years with JavaScript, I wanted to branch out into lower levels of 
programming. While researching starter projects I came across Robert Nystrom's wonderful 
book [*Crafting Interpreters*](https://craftinginterpreters.com/), and writing an 
interpreter in C sounded like a good way to go about that. However, copy-pasting code as 
I read the book lulled me into a passive learning state where I wasn't retaining the 
material. I tried typing each line of code instead, thinking that would force me to pay 
attention, but that just ended up being tedious.

For the book's [first half](https://craftinginterpreters.com/a-tree-walk-interpreter.html), 
using Python instead of Java helped me stay focused. I decided to take the same approach 
for the [second half](https://craftinginterpreters.com/a-bytecode-virtual-machine.html), 
but which alternative language should I use? Python didn't meet my lower-level goals. 
C++ seemed like a natural choice, but it was too difficult for me as a newcomer to quickly 
get up and running (I did eventually finish a PoC using C++ and plan to revisit it someday). 
Rust came up a lot during my search and not wanting to waste any more time, I thought, 
what the heck.

With an alternative host language selected, I began working through 
[clox](https://github.com/munificent/craftinginterpreters/tree/master/c) and started to 
wonder how the same techniques I was learning would apply to a different guest language. 
For whatever reason, a DSL I had used years ago when I was at ExactTarget/Salesforce named 
[AMPscript](https://developer.salesforce.com/docs/marketing/marketing-cloud/guide/ampscript.html) 
came to mind. Many super frustrating months later, I ended up with ETscript.

To be clear, this is a personal hobby project without a real-world application. Salesforce 
[doesn't offer a trial/developer account](https://ideas.salesforce.com/s/idea/a0B8W00000GdfIrUAJ/exacttarget-developer-edition) 
for their Marketing Cloud product (AMPscript's domain), so I had to rely on online resources 
and my fading memory of using it (I left in 2019). In other words, this is all experimental. 
Though 
[only a subset of functions](https://github.com/markgomez/etscript/tree/main/etscript-core#function-subset) 
are implemented, anyone who doesn't have Marketing Cloud access should still be able to 
get a feel for AMPscript's syntax by using this.

## Getting started

There isn't a fancy UI right now, so interaction takes place on the command line. Installing 
[Docker](https://www.docker.com/products/docker-desktop/) then running it using the 
included shell scripts will be the fastest way to try things out.

After installing Docker, download ETscript's source code then change into the repo's 
directory:

```bash
$ git clone https://github.com/markgomez/etscript
$ cd etscript
```

Run the OS-appropriate script. For Linux and macOS:

```bash
$ ./dev
```

For Windows [Terminal](https://learn.microsoft.com/windows/terminal/install) (recommended) 
or [PowerShell 7](https://learn.microsoft.com/powershell/scripting/install/installing-powershell-on-windows):

```pwsh
> .\dev.ps1
```

When the Docker image finishes building, a command prompt similar to the following should 
appear after the container starts:

```bash
etscript@3a60ddb1e9b3:/workspace$
```

ETscript can be run in at least two ways. One way is interactively using the REPL:

```bash
# build and run
$ cargo run
```

```bash
ETscript 0.1.1 (press Control-C to quit)
>>> %%=Add(2, 2)=%%
4
```

Another way is to run your source code from a plain text file (recommended). Create a new 
file, `hello.ets` (filename extension doesn't matter), in the working directory then 
copy-paste the following code into it:

```
%%[
var @you
set @you = "world"
]%%
Hello, %%=v(@you)=%%!
```

Then pass the source file to the `run` command:

```bash
# build and run
$ cargo run hello.ets

Hello, world!
```

To leave the container and return to the host shell, press Control-D or type `exit`:

```bash
etscript@3a60ddb1e9b3:/workspace$ exit
```

If you prefer not to use Docker, you'll need to install and set up the latest versions of 
the following before running the `cargo` commands:

- [Rust](https://www.rust-lang.org/tools/install)
- [.NET SDK 8](https://dotnet.microsoft.com/download/dotnet/8.0) (currently 
in preview)

Why version 8 and not 7? This project makes use of 
[.NET native AOT](https://learn.microsoft.com/dotnet/core/deploying/native-aot) and macOS 
support wasn't added until then.

## Why is .NET being used in some parts?

Many of AMPscript's functions for handling dates or formatting numbers and strings expect 
[.NET format specifiers](https://learn.microsoft.com/dotnet/standard/base-types/formatting-types). 
C#, along with external links to Microsoft's documentation, is referenced throughout 
[AMPscript's own documentation](https://developer.salesforce.com/docs/marketing/marketing-cloud/guide/Format.html). 
Date libraries are complicated things. I figured using date APIs that are native to .NET 
would be easier than trying to emulate them using 
[`time`](https://github.com/time-rs/time) or [`chrono`](https://github.com/chronotope/chrono).

## Are data extensions supported?

ETscript uses SQLite to emulate data extensions (tables). A table named `_test_table` is 
automatically created and can be used to try out the data extension functions. You can 
also create additional tables using your favorite SQL client, but you'll need to stick 
to the following expected typenames, some of which are custom<sup>*</sup>:

Typename | [Affinity](https://www.sqlite.org/datatype3.html#type_affinity)
:--- | :--- 
**Integer** | Integer
**Real** | Real
**Boolean**<sup>*</sup> | Numeric
**DateTime**<sup>*</sup> | Numeric
**Text** | Text

The database file, `etscript.db`, will be created (if it doesn't already exist) in the 
working directory (e.g., the repo's root).

A quick [`Lookup`](https://developer.salesforce.com/docs/marketing/marketing-cloud/guide/lookup.html) example:

```
%%[
UpsertData("_test_table", 1, "email", "me@example.com", "locale", "en-us")
]%%
Email: %%=Lookup("_test_table", "email", "locale", "en-us")=%%
```

```bash
$ cargo run hello.ets

Email: me@example.com
```


## What's still being worked on?

- Attribute scope
- Garbage collection
- Testing
- Documentation
