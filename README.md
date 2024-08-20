# Meloetta
A Discord app focused on temporary voice channels

## Setup
- Create an application through [Discord's Developer Portal](https://discord.com/developers) enabled with the "Server Members" gateway intent.
- Install [Rust](https://www.rust-lang.org/tools/install) and [PostgreSQL](https://www.postgresql.org/download/).
- Create a PostgreSQL database.
- Clone the repository.
- Copy the contents of the `.env.example` file into a new `.env` file and provide all variables.
- Run the app with `cargo run`. For a more optimized app, run the app with `cargo run --release`.