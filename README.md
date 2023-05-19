# SecretSync

Sync secrets between [Vault](https://www.vaultproject.io/) and a local `.env`.

## Installation

-   Binaries can be downloaded from the [releases page](https://github.com/rmarganti/scrtsync/releases).
-   Mac users can install via Homebrew:

    ```sh
    brew tap rmarganti/tap
    brew install scrtsync
    ```

## Syncing secrets between sources

```sh
scrtsync --from vault://secrets/you/secret/path --to file://.env
```

The `--from` and `--to` options can be any of the following:

-   `file://<path/to/your.env>` - Any .env file on your local file system.
-   `vault://<secretMountPath>/<path/to/your/secrets>` - A vault secret path.
    Note that `secretMountPath` is usually "secret" for most default configurations.

## Using presets

For convenience, you can define presets in a config file and then reference them on the command line.

1. First, create a config file by running `scrtsync init`.
2. This will create a `.scrtsync.json` file to your project's root directory. It is a JSON
   file that looks like the following:

    ```json
    {
        "$schema": "https://raw.githubusercontent.com/rmarganti/scrtsync/main/schemas/scrtsync.schema.1.0.0.json",
        "presets": {
            "pull": {
                "from": "vault://secrets/you/secret/path",
                "to": "file://.env"
            },
            "push": {
                "from": "file://.env",
                "to": "vault://secrets/you/secret/path"
            }
        }
    }
    ```

3. You can now run these presets by referencing them by name. To run the above,
   run either `scrtsync pull` or `scrtsync push`.
4. You can create and modify as many presets as are appropriate for your project.

## Options

| option     | description                                          |
| ---------- | ---------------------------------------------------- |
| `--config` | Path to a config file. Defaults to `.scrtsync.json`. |
