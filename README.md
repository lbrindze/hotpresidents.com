# hotpresidents.com
Code base for hotpresidents.com server

WIP...

To start service for local development, set the following Env Vars and run:

```
export DATA_LOAD_URI="..."
export API_KEY="..."
export SAVE_FILE="..."
export SAVE_TIMEOUT=30
export HOST_ADDRESS="localhost"
export HOST_PORT=8080

cargo run
```

Data is saved to a save file which is loaded during startup, this is configured with SAVE_FILE envvar.
to save counters send an http get request to `/save_state` and to reload the data after a change to the SAVE_FILE or the backing airtables base send an http get request to `/reload_data` or restart the server.
