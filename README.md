
# moin-dm

moin-dm is a display manager for linux that doesn't need a X-server but works on a console. moin-dm has no login features. It should normally be started after the user has been logged in e.g. by [greetd](https://git.sr.ht/~kennylevinsen/greetd).


## Compiling
To build moin-dm run `cargo build --release`. Afterwards you find the executable in `target/release/`.

## Frontends
Right now moin-dm only supports a simple frontend that prints the available setups/sessions to the console and allows the user to select one of them by entering the corresponding number. We will add an additional graphic cli later.
