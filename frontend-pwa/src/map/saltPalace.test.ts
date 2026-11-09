import { describe, expect, it } from "vitest";
import rooms from "../../../seed/kubecon_rooms.json";
import additionalRooms from "../../../seed/additional_rooms.json";
import { roomSpace, spaces } from "./saltPalace";

describe("architectural live room bindings", () => {
  it("does not invent a destination when the API has none", () => {
    expect(roomSpace()).toBeUndefined();
    expect(roomSpace("")).toBeUndefined();
    expect(roomSpace("unknown-room")).toBeUndefined();
  });
  it("places each seeded telemetry room exactly once", () => {
    for (const room of [...rooms, ...additionalRooms]) {
      expect(
        spaces.filter((space) => space.roomId === room.room_id),
      ).toHaveLength(1);
      expect(roomSpace(room.room_id)?.floor).toBeGreaterThanOrEqual(1);
    }
  });
});
