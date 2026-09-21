# Arca Mobile Privacy Policy

**Last updated: September 20, 2026**

Arca is a local-first password manager developed by Adrian Kucharczyk. This
policy covers the Arca iOS application.

## Data collection

Arca does not collect, transmit, sell, or share personal data. The app has no
analytics, advertising, telemetry, account service, or remote crash-reporting
SDK. The developer cannot access your vault, master password, entries, usage,
or device clipboard through the app.

## Data stored on the device

Arca processes KDBX vault files on your device. Vault files remain encrypted
at rest. You choose a file through the iOS document picker or create one in
Arca's app storage.

Arca may store:

- security-scoped file bookmarks in iOS Keychain so you can reopen recent
  vaults;
- theme, auto-lock timeout, clipboard timeout, and the recent-vault preference
  in app-only `UserDefaults`;
- the encrypted KDBX file at the location you select.

Arca does not persist the master password, decrypted vault contents, revealed
passwords, generated passwords, notes, URLs, or vault metadata in
`UserDefaults`. Unlocked data is cleared on lock and protected lifecycle
transitions.

## Document providers and backups

If you select a file exposed by iCloud Drive or another Files provider, that
provider handles the file according to its own terms and privacy policy. Arca
does not include or communicate with cloud-provider SDKs. Device backups and
provider backups are controlled by you, iOS, and the provider you selected.

## Clipboard

Arca writes a password to the iOS clipboard only after an explicit copy action.
It asks iOS to expire the value after the selected interval and clears it when
possible while Arca still owns it. iOS, a keyboard, a clipboard manager, or
another app may retain a copied value outside Arca's control.

## Retention and deletion

Vault data remains until you delete the KDBX file from its storage location.
You can remove recent file handles in Arca settings. Deleting the app removes
its app container and preferences; files stored by an external document
provider may remain and must be deleted through that provider.

## Children

Arca is a general-purpose utility and does not knowingly collect data from
children or anyone else.

## Changes

Material changes to Arca's data handling will be reflected in this policy and
the App Store privacy answers before the corresponding release.

## Support and privacy questions

Open a public support issue at <https://github.com/akei9/arca/issues>. Report a
security vulnerability privately at
<https://github.com/akei9/arca/security/advisories/new>.
