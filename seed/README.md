# VECTOR demo data

The base dataset has 12 rooms, 25 sessions, and 12 attendee profiles. Supplemental fixtures add 12 rooms, 24 sessions, and 24 fictional attendees, for 24 rooms, 49 sessions, and 36 seeded profiles. Real OAuth users are additional records.

- `additional_rooms.json`: mapped meeting rooms across all three floors, including quiet rooms 203A and 355A.
- `additional_talks.json`: fictional demo sessions in the existing time windows. Quiet spaces remain unscheduled.
- `additional_builders.json`: fictional attendees labeled “Demo,” with local full-color SVG monograms under `frontend-pwa/public/avatars/`. They do not claim real GitHub identities (`github_id` is zero); signing in still requires verified GitHub OAuth.

Startup creates missing indices and seeds the base dataset only when an index is empty. It then backfills supplemental fixtures with Elasticsearch bulk **create** operations. Existing IDs return a conflict and are retained. This makes repeated startup safe for signed-in profiles, changed room readings, and local demo edits. Editing a supplemental fixture does not overwrite an existing document; new IDs are added on the next startup.

Missing talk/profile embeddings are generated deterministically for the demo dataset. The seed data is illustrative, not an official conference program or live venue measurement. No handshakes or follow-up messages are fabricated.

Quiet spaces must have no active talk, less than 20% occupancy, and less than 45 dB. The map and sensory query use those same thresholds. Availability changes reactively as telemetry arrives.
