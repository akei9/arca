---
status: accepted
date: 2026-09-08
decision-makers: ["akei9"]
related: ["#138", "#207", "#210", "#211", "#212", "#213"]
---

# ADR-0013: Browser extension autofill threat model and permissions

## Status History

- 2026-09-08 - proposed by Codex for maintainer review.
- 2026-09-08 - accepted by maintainer approval.

## Context

ADR-0007 defines the browser extension as a thin UI and autofill client.
ADR-0009 keeps browser access local-first through native messaging to the
desktop app or a small native host. ADR-0011 defines the first extension
bootstrap plan and maps one-time approved fill to the `CopySecret` capability,
not to a general `RevealSecret` surface.

Autofill is useful because it places credentials into web pages. That also
makes it the highest-risk extension workflow. The extension observes page
structure, receives browser origin context, talks to a trusted native boundary,
and may briefly handle a plaintext password for a user-approved fill. This ADR
defines the autofill threat model and browser permission policy before
implementation.

## Decision Drivers

- Keep extension permissions narrow and explainable.
- Treat tabs, frames, forms, origins, page scripts, and DOM content as
  untrusted.
- Require explicit user action before copying or injecting a secret.
- Avoid storing or logging plaintext secrets, page contents, form contents, or
  full browsing history.
- Preserve `vault-api` client capability enforcement at the bridge.

## Assets

The extension threat model protects:

- master password and vault keys, which must never enter the extension;
- unlocked vault session state, owned by the native side;
- plaintext entry passwords and one-time fill payloads;
- vault metadata, including titles, usernames, URLs, tags, and collection names;
- native messaging protocol integrity and capability decisions;
- user browsing context, including active tab URL, frame origin, and page form
  shape.

## Trust Boundaries

The extension has these boundaries:

- Browser UI surfaces: popup, options page, background service worker, and
  content scripts are trusted only as extension code, not as vault owners.
- Native side: the desktop app or native host owns unlock, KDF work, KDBX
  parsing, vault file access, session lifetime, and host-owned clipboard writes.
- Web pages: top-level pages, iframes, page scripts, DOM attributes, form
  fields, autofill hints, and page events are untrusted input.
- Browser APIs: tab URL, frame origin, extension permissions, and native
  messaging ports are security-relevant platform signals, but still require
  validation and error handling.
- `vault-api`: `ClientKind::BrowserExtension` and capability checks are the
  policy source of truth for bridge operations.

Content scripts run in an isolated extension world when the browser provides
one, but page-controlled DOM and events remain attacker-controlled data.

## Threats

The first implementation must account for:

- phishing pages that resemble a stored service but use a different origin;
- lookalike domains, public suffix tricks, and deceptive subdomains;
- login forms embedded in third-party iframes;
- first-party pages that include third-party login iframes;
- hostile page scripts that mutate fields after detection or observe injected
  values;
- hidden, off-screen, disabled, or decoy password fields;
- page-driven requests that try to trigger fill without user approval;
- compromised or confused extension contexts that request broader capabilities;
- malformed native messaging payloads or version mismatches;
- logs, diagnostics, screenshots, crash reports, or telemetry that accidentally
  retain secrets or page content.

## Permission Policy

The first extension MVP should request only these browser permissions:

| Permission | Required | Justification | Constraints |
| --- | --- | --- | --- |
| `nativeMessaging` | Yes | Connect to the local desktop app or native host that owns vault access. | Only the reviewed Arca host name is allowed. Host install and update paths need release checks. |
| `storage` | Yes | Store non-secret extension settings, diagnostics preferences, and bounded metadata cache state. | Never store plaintext secrets, unlocked vault snapshots, master passwords, form contents, or browsing history. Clear session cache on lock, disconnect, origin change, and version change. |
| `activeTab` | Yes | Read the active tab context after a user invokes the extension and scope an approved fill/copy action to that tab. | Do not replace with persistent `tabs` permission for the MVP. |
| `host_permissions` for `http://*/*` and `https://*/*` | Yes | Run content scripts on normal web pages to detect fillable username/password forms and report redacted form shape. | Exclude browser-internal, extension, file, and local privileged schemes. Do not request `<all_urls>` unless a later ADR justifies non-http(s) support. |

The MVP should not request these permissions:

- `tabs`, unless a later implementation proves `activeTab` and content-script
  origin reporting are insufficient;
- `cookies`;
- `history`;
- `webRequest` or `declarativeNetRequest`;
- `clipboardRead`;
- `clipboardWrite`, because the native side should own copy operations;
- broad file access or access to `file://` URLs;
- arbitrary optional host permissions beyond reviewed http/https content-script
  matches.

Any new permission requires a PR that updates this ADR or supersedes it, adds a
store-review justification, and explains what user-visible workflow breaks
without the permission.

## Origin And Matching Policy

The extension must derive fill decisions from normalized origins, not visual
page text. Matching should compare the selected entry URL origin against the
active top-level origin and, when filling an iframe, the target frame origin.

For the MVP:

- exact scheme and registrable-domain matches are allowed;
- `https` entries may fill only `https` pages unless the user explicitly
  approves an insecure-origin warning for that single action;
- subdomain matches are allowed only when the stored entry or user choice
  intentionally covers that subdomain relationship;
- public suffixes, bare TLDs, IP literals, localhost, and private network names
  require explicit handling and tests before support is advertised;
- lookalike, punycode, mixed-script, and confusable domains should produce a
  warning or fail closed until a tested policy exists;
- page titles, logos, favicons, placeholder text, and field labels are hints,
  not identity.

The extension may show candidate entries for user selection, but it must not
auto-select a credential solely from page-provided labels or hidden fields.

## Frames And Forms

The content script may detect candidate fields and send redacted form shape to
the background service worker. Redacted form shape may include field role,
visibility state, frame origin, top-level origin, and stable field handles for
that page session. It must not include typed user input, existing field values,
full DOM HTML, page text, screenshots, or arbitrary attributes.

Filling an iframe is allowed only when:

- the target frame origin is known;
- the user can see which origin will receive the secret;
- the selected entry is valid for that frame origin or the user approves a
  single-action warning;
- the frame remains present and visible at fill time.

The extension must fail closed for hidden password fields, invisible frames,
sandboxed frames without a trustworthy origin, cross-origin ambiguity, detached
fields, forms that change between approval and fill, or multiple candidate
password fields without a clear user-selected target.

The extension must not submit forms automatically in the MVP.

## Secret Exposure Rules

Passwords may be copied or injected only after explicit user action in the
extension UI or a browser-presented extension affordance. Page scripts must not
be able to trigger secret movement by dispatching DOM events, changing field
names, focusing fields, or sending messages to content scripts.

The extension must not expose a general reveal surface. For autofill, the
native side may return a one-time fill payload only after:

- the native bridge verifies `ClientKind::BrowserExtension`;
- `vault-api` authorizes the `CopySecret` capability;
- the request includes protocol version, correlation ID, active tab origin,
  target frame origin, selected entry ID, and user approval state;
- the origin policy passes or the user accepts a single-action warning.

The content script must write the one-time payload to the selected fields, drop
it immediately, and send only non-secret completion or failure status back to
the background service worker. The extension must not cache fill payloads,
retain them for retry, write them to browser storage, log them, or include them
in diagnostics.

Copy fallback should prefer native-side clipboard writes with the same explicit
user action and clearing behavior as the desktop app. Extension-side clipboard
writes are out of scope for the MVP.

## Audit, Logging, And Diagnostics

Logs and diagnostics may include:

- extension version;
- native host version;
- protocol version;
- correlation ID;
- operation name;
- coarse status such as connected, locked, unlocked, denied, filled, copied, or
  failed;
- normalized origin hashes or redacted origins only when needed for debugging
  and approved by a later implementation policy.

Logs and diagnostics must not include:

- plaintext secrets;
- master password material;
- vault keys;
- full entry metadata snapshots;
- typed form values;
- page text or full DOM HTML;
- screenshots;
- complete browsing history;
- native messaging payloads that contain secret-bearing fields.

The native side and extension side should share correlation IDs that are safe to
log, but neither side should log raw secret-bearing requests or responses.

## Store Disclosure

The extension store listing, review notes, and privacy policy must explain:

- the extension connects to a local Arca desktop app or native host;
- vault files, unlock, and KDF work stay outside the browser extension;
- credentials are filled or copied only after user action;
- content scripts inspect login form structure on http/https pages;
- browser storage is used only for non-secret settings and bounded non-secret
  state;
- no plaintext passwords, master password, vault contents, page contents, or
  browsing history are sold, synced, or retained by Arca.

## Consequences

### Positive

- Makes extension autofill reviewable before implementation.
- Gives store permissions a concrete, security-driven rationale.
- Keeps secret movement user-driven and mapped to the existing capability
  model.
- Reduces accidental retention of page contents, browsing context, and
  plaintext secrets.

### Negative

- Broad http/https content-script reach is still sensitive and must be clearly
  justified in stores.
- No silent autofill or auto-submit means the MVP is less magical.
- Lookalike and iframe warnings add product work before release.

### Neutral

- A later ADR may expand permissions or matching behavior after implementation
  tests prove the need.

## Compliance

- [ ] Add browser extension permission documentation before store submission.
- [ ] Add native messaging tests that deny missing or mismatched client kind,
  protocol version, active tab origin, target frame origin, or capability.
- [ ] Add autofill tests for exact origin, subdomain, iframe, insecure origin,
  hidden field, detached field, and lookalike-domain handling.
- [ ] Add log/diagnostic tests or review checks proving secret-bearing payloads,
  form values, DOM text, and page screenshots are not retained.
- [ ] Keep copy/fill mapped to `CopySecret` unless a later ADR changes the
  capability policy with tests.

## Revisit / Out Of Scope

- Creating `arca-extension`.
- Store release checklist details covered by #212.
- Safari extension support.
- Automatic form submission.
- Browser-side password reveal UI.
- Extension-side clipboard writes.
- Sync, accounts, pairing, remote unlock, or shared vaults.

## References

- #138
- #207
- #210
- #211
- #212
- #213
- Chrome Extensions permissions documentation:
  https://developer.chrome.com/docs/extensions/develop/concepts/declare-permissions
- Chrome Extensions native messaging documentation:
  https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging
- MDN WebExtensions native messaging documentation:
  https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging
