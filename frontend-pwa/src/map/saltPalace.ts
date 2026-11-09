// Architectural schematic rebuilt from the supplied Salt Palace diagram.
// Coordinates describe the illustration, not a surveyed navigation system.
export type Floor = 1 | 2 | 3;
export interface MapSpace {
  id: string;
  label: string;
  floor: Floor;
  kind: "hall" | "meeting" | "ballroom" | "lounge";
  d: string;
  x: number;
  y: number;
  roomId?: string;
  approximate?: boolean;
}
const rect = (
  id: string,
  label: string,
  floor: Floor,
  kind: MapSpace["kind"],
  x: number,
  y: number,
  w: number,
  h: number,
  roomId?: string,
): MapSpace => ({
  id,
  label,
  floor,
  kind,
  d: `M${x} ${y}h${w}v${h}h-${w}Z`,
  x: x + w / 2,
  y: y + h / 2,
  roomId,
});
export const spaces: MapSpace[] = [
  ...["E", "D", "C", "B", "A"].map((name, i) =>
    rect(
      `hall-${name.toLowerCase()}`,
      `HALL ${name}`,
      1,
      "hall",
      60 + i * 88,
      485,
      88,
      150,
    ),
  ),
  {
    id: "hall-5",
    label: "HALL 5",
    floor: 1,
    kind: "hall",
    d: "M480 145H575V65H685V280H480Z",
    x: 590,
    y: 204,
  },
  rect("hall-4", "HALL 4", 1, "hall", 480, 280, 205, 105, "hall-4-keynote"),
  rect("hall-3", "HALL 3", 1, "hall", 525, 385, 105, 50),
  rect("hall-2", "HALL 2", 1, "hall", 525, 435, 105, 50),
  {
    id: "hall-1",
    label: "HALL 1",
    floor: 1,
    kind: "hall",
    d: "M500 485H685V595H560L530 635H500Z",
    x: 592,
    y: 540,
  },
  ...["J", "H", "F", "D", "B", "I", "G", "E", "C", "A"].map((name, i) =>
    rect(
      `ballroom-${name.toLowerCase()}`,
      name,
      1,
      "ballroom",
      205 + (i % 5) * 34,
      720 + Math.floor(i / 5) * 46,
      34,
      46,
      name === "B" ? "ballroom-b" : undefined,
    ),
  ),
  rect("155b", "155B", 1, "meeting", 738, 530, 47, 45, "room-155b"),
  ...["155A", "155C", "155D", "155E", "155F"].map((name, i) =>
    rect(
      name,
      name,
      1,
      "meeting",
      690 + (i % 3) * 48,
      485 + Math.floor(i / 3) * 90,
      47,
      44,
    ),
  ),
  rect("150", "150", 1, "meeting", 425, 720, 65, 78, "room-150"),
  rect("151", "151", 1, "meeting", 505, 720, 65, 78, "room-151"),
  {
    ...rect(
      "south-nook",
      "SOUTH FOYER",
      1,
      "lounge",
      70,
      650,
      120,
      48,
      "lounge-sensory-south",
    ),
    approximate: true,
  },
  rect("250a", "250A", 2, "meeting", 695, 700, 68, 58, "room-250a"),
  rect("251c", "251C", 2, "meeting", 765, 760, 68, 58, "room-251c"),
  rect("250b", "250B", 2, "meeting", 765, 700, 68, 58),
  rect("251a", "251A", 2, "meeting", 695, 760, 68, 58),
  rect("204b", "204B", 2, "meeting", 810, 420, 70, 48, "room-204b-quiet"),
  ...["203A", "203B", "204A"].map((name, i) =>
    rect(name, name, 2, "meeting", 810, 260 + i * 52, 70, 48),
  ),
  ...["255A", "255B", "255C", "255D", "255E", "255F"].map((name, i) =>
    rect(
      name,
      name,
      2,
      "meeting",
      902 + (i % 3) * 28,
      488 + Math.floor(i / 3) * 48,
      27,
      47,
    ),
  ),
  {
    ...rect(
      "north-lounge",
      "NORTH LOUNGE",
      2,
      "lounge",
      810,
      180,
      70,
      65,
      "lounge-sensory-north",
    ),
    approximate: true,
  },
  {
    ...rect(
      "community",
      "COMMUNITY",
      2,
      "lounge",
      695,
      625,
      138,
      50,
      "room-mezz-community",
    ),
    approximate: true,
  },
  ...["355A", "355B", "355C", "355D", "355E", "355F"].map((name, i) =>
    rect(
      name,
      name,
      3,
      "meeting",
      902 + (i % 3) * 28,
      260 + Math.floor(i / 3) * 48,
      27,
      47,
      name === "355E" ? "room-355e" : undefined,
    ),
  ),
];
export const roomSpace = (id?: string) =>
  id ? spaces.find((space) => space.roomId === id) : undefined;
export const floorNames: Record<Floor, string> = {
  1: "First floor",
  2: "Second floor",
  3: "Third floor",
};
