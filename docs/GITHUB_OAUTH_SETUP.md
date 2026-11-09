# Enable real GitHub sign-in for VECTOR

This app uses GitHub's OAuth authorization-code flow with PKCE. GitHub verifies the account; VECTOR then sets a signed, HttpOnly, SameSite=Lax session cookie. Cookies are Secure when `PUBLIC_BASE_URL` uses HTTPS. The app requests `read:user`, reads public repositories for recommendations, and does not ask for broad private-repository access. GitHub access tokens are used during sign-in and are not stored in browser storage or the session cookie.

## 1. Register the GitHub OAuth App

1. Sign into the GitHub account that should own VECTOR's integration.
2. Open https://github.com/settings/developers.
3. Choose **OAuth Apps**, then **New OAuth App** (or **Register a new application**).
4. Enter:

   | Setting | Value |
   | --- | --- |
   | Application name | `VECTOR` |
   | Homepage URL | `https://vector.winslow.plus` |
   | Description | `Personal conference map and session recommendations` |
   | Authorization callback URL | `https://vector.winslow.plus/api/auth/github/callback` |

5. Leave **Enable Device Flow** off. VECTOR uses browser sign-in.
6. If GitHub offers **Expire user access tokens**, it can stay enabled: VECTOR uses the token only during the callback and does not keep it for background access or refresh.
7. Click **Register application**.

These steps follow [GitHub's OAuth App registration guide](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/creating-an-oauth-app).

## 2. Copy the credentials into the server environment

1. On the OAuth App settings page, copy the **Client ID**.
2. Click **Generate a new client secret**, and complete any GitHub confirmation.
3. On the VECTOR host, edit the existing environment file:

   ```sh
   cd /home/server_8989320/vector
   nano .env
   ```

4. Set these entries, replacing the example placeholders locally:

   ```dotenv
   PUBLIC_BASE_URL=https://vector.winslow.plus
   GITHUB_CLIENT_ID=YOUR_CLIENT_ID
   GITHUB_CLIENT_SECRET=YOUR_CLIENT_SECRET
   VECTOR_ADMIN_USERS=YOUR_GITHUB_HANDLE
   ```

   `VECTOR_ADMIN_USERS` is a comma-separated list of accounts that may run crowd-surge, reset, check-in, and audit controls. Ordinary signed-in attendees can use their maps, schedules, quiet mode, badges, and connections. Do not prefix handles with `@`.

5. Keep the existing private `SESSION_SECRET` if it is a securely generated value of at least 32 bytes. If it needs replacing, generate a value with `openssl rand -hex 32` and paste it into `SESSION_SECRET=` in `.env`. Replacing it signs everyone out. Do not copy the sample secret from `.env.example` into production.
6. Save the file. Keep it private (`chmod 600 .env` if appropriate for your host).

Do not put the client secret in frontend files, a `VITE_` variable, Git, a URL, or chat. `.env` is ignored by Git and excluded from the Docker build context. The client ID alone is not a password.

## 3. Restart only the Vector API

Environment changes take effect when the API starts. The app currently runs as a foreground development process; it does not yet have a managed service.

1. Stop the existing `backend-api` process from the terminal where it was started. If it was started by this agent, ask the agent to restart it after saving `.env`; there is no need to restart WSL, Tailscale, Elasticsearch, or the host.
2. From `/home/server_8989320/vector`, start the verified binary:

   ```sh
   ./target/debug/backend-api
   ```

   It loads `.env` itself. If rebuilding is needed on this host, use one Cargo job and the existing low-memory wrapper:

   ```sh
   CARGO_BUILD_JOBS=1 RUSTC_WRAPPER=/tmp/vector-rustc-low-memory cargo build -p backend-api
   ```

Keep the API terminal running. Durable systemd/container service setup is separate from OAuth configuration.

## 4. Verify configuration and sign in

1. Open `https://vector.winslow.plus/api/auth/status`. Expect `"oauth_configured": true`. This reports that settings exist; it does not prove the client secret is valid.
2. Open `https://vector.winslow.plus` in a private browser window. You should see the architectural map preview and **Sign in with GitHub**. Attendee data must not appear yet.
3. Click **Sign in with GitHub**. Check that the authorization page is on `github.com`, then authorize VECTOR.
4. GitHub returns you to `/api/auth/github/callback`; VECTOR verifies state and PKCE, fetches your verified GitHub identity, builds your profile, and redirects to `/`.
5. Confirm the account strip shows **your own GitHub handle**. Wait for the live-radar connection, select rooms, and try changing floors.
6. Reload. You should stay signed in. In browser developer tools under Application/Storage → Cookies, `vector_sid` should be **HttpOnly**, **Secure**, and **SameSite=Lax**. Temporary `vector_oauth_state` and `vector_oauth_verifier` cookies should have been cleared.
7. Click **Sign out**. Attendee data should disappear. Reloading should show the sign-in screen. Old impersonation cookies and `X-Vector-User` no longer authenticate.

The signed session lasts seven days. VECTOR uses essential authentication cookies and does not add advertising cookies. No cookie consent banner is implemented.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| `oauth_configured: false` | Both GitHub credentials and a signing key of at least 32 bytes must be present. Restart the API after editing `.env`. |
| GitHub reports redirect mismatch | The app callback must exactly match `https://vector.winslow.plus/api/auth/github/callback`, with the same host and scheme. `PUBLIC_BASE_URL` should have no path. |
| OAuth state expired/missing/mismatched | Start again from the sign-in button in the same browser. Allow essential cookies and avoid switching between localhost and the public domain during one flow. |
| GitHub rejects the authorization code | Verify the Client ID/secret belong to this OAuth App. Start a new flow; codes are short-lived and single-use. |
| Sign-in fails while creating the profile | GitHub profile/repo reads, Elasticsearch, and the profile pipeline must be reachable. Identity verification never falls back to a demo account. |
| Surge/reset controls are absent | Add your verified GitHub username to `VECTOR_ADMIN_USERS`, restart the API, and refresh. |
| Cookies work on HTTPS but not HTTP localhost | Secure cookies are not intended for ordinary HTTP origins. For local development use a separate OAuth App and matching `PUBLIC_BASE_URL`, frontend origin, and callback. |
| Unexpected old UI | Reload online and let the service worker update. API responses are never cached. |

GitHub's [OAuth flow documentation](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps) describes state, PKCE, callback handling, and identity verification. Automated tests validate these against a local provider mock; completing the real GitHub consent flow requires your credentials and browser.

## Mobile recording flow

1. Sign in on your phone with a handle listed in `VECTOR_ADMIN_USERS`.
2. Keep portrait orientation. The attendee app uses one column; the desktop telemetry dock is hidden.
3. Use floor buttons or **Find your room** to explore. Search results provide full-size touch targets for small meeting-room shapes; **Find a quiet space** lists available quiet rooms across floors. Zoom buttons and drag let you inspect the SVG.
4. Open **Demo controls**, then **Trigger surge**. The panel collapses so the map and new recommendation remain visible. Reopen it to **Reset surge** for another take.
5. Toggle **Quiet** to show the sensory haven, then open the QR badge or saved connections as needed.

These layouts and controls are verified in a 390 × 844 browser viewport. A final sign-in and recording check on your actual phone remains useful after the GitHub credentials are configured.
