# Sign-in/profile timeout during demo preparation

The public API logged repeated reqwest timeouts reading and writing `builders/_doc/DevTheDev189` and reading `rooms_state` from the primary Elasticsearch at `127.0.0.1:9200`. OAuth callback attempts therefore failed even though GitHub API connectivity returned 200.

The WSL host has about 2.8 GiB RAM. It was also running a second Elasticsearch inside a k3d cluster and unrelated application containers. Available memory was about 0.8 GiB, swap usage about 3.2 GiB, and the duplicate Java process used about 1.3 GiB RSS. Profile/state requests exceeded the API HTTP timeout; public logs showed roughly 10–28 second failures.

With the user's explicit authorization, stopped the unrelated Docker containers and the k3d stack. Kept `vector-elasticsearch`, its volumes, the public API, WSL, and Tailscale. No data volumes or containers were deleted.

Available memory recovered to about 2.2 GiB. Verification against the same services returned: Elasticsearch cluster health 200 in 0.097 s, the user's profile document 200 in 2.530 s on the first recovered read, room count 200 in 0.395 s, and authenticated state on the isolated test API 200 with 24 rooms in 1.367 s. Public health and OAuth configuration endpoints returned 200. Retry login from the home page rather than reusing an expired OAuth callback URL.

This diagnosis used local process/container inspection, application logs, and HTTP probes. It was not a Dash0 Agent0, Radar AI, or Gitar finding and must not be presented as one. Do not restart the stopped workloads before the recording/deadline without checking memory headroom.
