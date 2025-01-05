# HackTheCrous.crawler

A script to aliment [Hack the Crous](https://hackthecrous.com) from crous's website. Btw it uses Rust now.

## Prerequisites

- cargo
- an up and running postgresql database

## Getting started (quick)

Fill a .env file for local dev

```
DATABASE_URL=
LOKI_ENDPOINT=
```

⚠️  the `LOKI_ENDPOINT` is optional. If it is not set, the logger will fallback to the default `tracing` logger.


And then execute : 

```bash
cargo run -- <your-action>
```

Please refer to the next section to get better understanding of all available actions.

## Commands

available actions are :

- restaurants -> scrape restaurants from the given restaurant
- up -> run the migrations
- meals -> scrape meals on all restaurants available in the given database
- bootstrap -> calls every actions up -> restaurants -> meals, so in one action you can bootstrap a new database with all needed data
