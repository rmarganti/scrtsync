# SecretSync

Sync secrets between [Vault](https://www.vaultproject.io/) and a local `.env`.

## Installation

- Binaries can be downloaded from the [releases page](https://github.com/rmarganti/scrtsync/releases).
- Mac users can install via Homebrew:

  ```sh
  brew tap rmarganti/tap
  brew install secret-sync
  ```

## Syncing secrets between sources

```sh
scrtsync --from vault://secrets/you/secret/path --to file://.env
```

The `--from` and `--to` options can be any of the following:

- `file://<path/to/your.env>` - Any .env file on your local file system.
- `vault://<secretMountPath>/<path/to/your/secrets>` - A vault secret path.
  Note that `secretMountPath` is usually "secret" for most default configurations.
- `k8s://<context>/<secretName>` - A Kubernetes secret.

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

| option     | description                                                          |
| ---------- | -------------------------------------------------------------------- |
| `--config` | Path to a config file. Defaults to `.scrtsync.json`.                 |
| `--diff`   | Show a diff between `--from` and `--to` without writing any secrets. |

## Development

### Preparing a Release

1. Trigger the "Prepare Release" workflow manually from the Actions tab
   - This automatically bumps the version, updates the CHANGELOG, and creates a git tag

### Publishing a Release

1. The "Build and Publish" workflow runs automatically when a release is published
   - Builds binaries for Linux (x86_64, aarch64) and macOS (x86_64)
   - Publishes binaries to the GitHub Release
   - Automatically updates the Homebrew formula in `rmarganti/homebrew-tap`

Users can then install the new version via:
```sh
brew tap rmarganti/tap
brew install secret-sync
```
