# Releasing

For the maintainer. A release is a tag: pushing `vX.Y.Z` builds every
platform, signs and notarizes the macOS binary, app and disk image,
publishes the GitHub release, deploys the browser version, and updates
the Homebrew tap. This page covers the setup that has to be done once,
and how to check a change to the release without publishing anything.

## One-off setup

Six repository secrets drive the release. Set each with
`gh secret set NAME -R abedegno/gorilla-rust`, which reads the value from
standard input, so that it never lands in your shell history.

### A Developer ID certificate

Only the account holder of the Apple Developer team can create one.

1. In Keychain Access, choose Keychain Access > Certificate Assistant >
   Request a Certificate From a Certificate Authority. Enter your email,
   choose "Saved to disk", and save the `.certSigningRequest`.
2. At <https://developer.apple.com/account/resources/certificates/add>,
   choose **Developer ID Application**, upload the request, and download
   the `.cer`. Double-click it to add it to your login keychain.
3. In Keychain Access, under My Certificates, find "Developer ID
   Application: …", right-click it, and choose Export. Save it as a
   `.p12` with a strong password. The export includes the private key,
   which is what signing needs.
4. Set the two secrets:

   ```sh
   base64 -i DeveloperID.p12 | gh secret set MACOS_CERTIFICATE_P12 -R abedegno/gorilla-rust
   gh secret set MACOS_CERTIFICATE_PASSWORD -R abedegno/gorilla-rust   # paste the password
   ```

5. Delete the `.p12` file. The certificate stays in your keychain.

The certificate lasts five years.

### An App Store Connect API key, for notarization

1. At <https://appstoreconnect.apple.com/access/integrations/api>, open
   **Team Keys** and add a key named `gorilla-rust notary` with the
   **Developer** role.
2. Download the `AuthKey_XXXXXXXXXX.p8`. Apple lets you download it once
   only.
3. Set the three secrets. The key ID is in the key's row, and the issuer
   ID is at the top of the page:

   ```sh
   base64 -i AuthKey_XXXXXXXXXX.p8 | gh secret set APPLE_API_KEY_P8 -R abedegno/gorilla-rust
   gh secret set APPLE_API_KEY_ID -R abedegno/gorilla-rust
   gh secret set APPLE_API_ISSUER_ID -R abedegno/gorilla-rust
   ```

4. Delete the `.p8` file.

The key does not expire, but it can be revoked on the same page.

### The Homebrew tap

The formula and cask live in `abedegno/homebrew-tap`, which the release
writes to.

1. Create it, with a README so that it has a default branch:

   ```sh
   gh repo create abedegno/homebrew-tap --public \
     --description "Homebrew formula and cask for gorilla-rust" --add-readme
   ```

   Then replace the README with the one in "The tap's README" below.
2. At <https://github.com/settings/personal-access-tokens/new>, create a
   fine-grained token. Name it `gorilla-rust tap`, give it access to
   **only** `abedegno/homebrew-tap`, set **Contents** to "Read and
   write", and give it the longest expiry offered.
3. `gh secret set TAP_TOKEN -R abedegno/gorilla-rust`, and paste the
   token.
4. Put a reminder in your calendar for a week before it expires.

### The tap's README

    # abedegno/homebrew-tap

    Homebrew packages for [gorilla-rust](https://github.com/abedegno/gorilla-rust).

        brew install abedegno/tap/gorilla-rust      # the gorilla-rust command
        brew install --cask abedegno/tap/gorillas   # Gorillas.app

    `Formula/gorilla-rust.rb` and `Casks/gorillas.rb` are written by
    gorilla-rust's release workflow on each release. Don't edit them here;
    change the templates in gorilla-rust's `packaging/homebrew/` instead.

## Checking a change to the release

Run the Release workflow by hand, from the branch that changes it:

```sh
gh workflow run release.yml --ref BRANCH -f ref=BRANCH
```

`--ref` runs the workflow file from that branch, not from `main`. A manual
run builds, signs and notarizes everything (if the secrets are set), and
renders and audits the Homebrew files. It uploads all of it as workflow
artifacts. It never publishes a release and never pushes to the tap.
Download the macOS artifact and open the DMG to see what a user would get.

## Cutting a release

Follow "Releasing" in `CONTRIBUTING.md`. After the tag's run, the
`homebrew` job has pushed the new formula and cask, and installed both
from the tap to check them.

## When something expires

- **The certificate** (after five years): the macOS job fails at "Import
  the signing certificate" or at `codesign`. Make a new one as above and
  replace both certificate secrets.
- **The API key** (only if revoked): notarization fails with an
  authentication error. Make a new key and replace the three key secrets.
- **The tap token** (after its expiry): the `homebrew` job fails at "Push
  to the tap". Make a new token and replace `TAP_TOKEN`, then re-run the
  failed job from the run's page. Re-running is safe; see below.

Re-running the `homebrew` job for a version the tap already has changes
nothing: it only commits when the files differ.
