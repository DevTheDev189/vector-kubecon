# Schedule-first attendee experience

The user confirmed GitHub sign-in works and wants the next iteration to focus on frontend UX. The current page leads with account controls and a large room-search panel, pushing the recommended day below the map. The new flow starts with the attendee's day, then puts the map in context, and ends with their profile and nearby peers.

## First-pass layout

1. Compact attendee header: VECTOR identity, live connection state, Quiet and QR actions. Account details move into the lower profile section. Admin controls become one collapsed section near the bottom.
2. **Your recommended day**: a compact chronological calendar list with start/end times, recommended session titles and room labels. Show the actual conference date and Mountain Time. Highlight the selected session; do not label future seed times as “now.” Session selection updates the map. Keep the selected session's detailed capacity, bookmark, and match rationale available in a compact disclosure.
3. **Map**: architectural map tied to the selected session. Replace always-open search with a clear **Find your room** button. Keep a compact **Quiet spaces** shortcut and bright quiet-room markers. Search works across floors and closes after choosing a room. Looking up a room does not change the selected calendar session. Offer a clear way back to the session on the map.
4. **You and people like you**: personal builder summary, optional tags/icebreaker, and nearby peers. Keep refresh/sign-out in an account disclosure. Handshake and saved connections remain available.
5. Admin demo controls are collapsed, mobile accessible, and below the primary attendee flow. Retain the desktop proof dock for recording/judging.

## Interaction details to preserve

- Selection is shared between calendar and map; search can temporarily inspect a different room without mutating the agenda.
- A surge changes the affected session recommendation and exposes an explanation, not just a color change. Quiet mode retains a visible haven and clear fallback if none is available.
- Keep 44px touch targets, 16px search input, keyboard focus/escape handling, no horizontal phone overflow, full-color avatars, readable color contrast, and reduced-motion support.
- Treat unbookmarked sessions as recommendations rather than reservations. Bookmark actions retain their existing backend meaning.
- No backend, OAuth, database, or seed changes are planned for this iteration.

## Validation

Build/typecheck and frontend state tests, then browser-check calendar-to-map selection, collapsed search and cross-floor lookup, return-to-session action, quiet discovery, surge/reset, QR/handshake/vault, sign-out, offline shell, keyboard dismissal, and 390px layout. Review screenshots before publishing.

Confirmed by the user: compact full-day calendar.

Reference: https://www.nngroup.com/articles/progressive-disclosure/
