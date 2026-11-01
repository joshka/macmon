# TUI design principles

What the TUI optimizes for, and the rules that follow from it. Each rule names the screenshot
evidence in [#114](https://github.com/vladkens/macmon/issues/114) where it came from.

The TUI answers three questions at a glance, in this order: how hard is the chip working, what is
it costing in power and heat, and which processes are responsible. Everything on screen serves one
of those or gets out of the way.

## 1. One meaning per color

- **Identity colors name a subsystem**, the same everywhere it appears: CPU green (E cores teal in
  RGB), GPU purple, ANE amber, memory blue. A metric's load graph, power graph, title and table
  column header share its color.
- **Green → amber → red means load**, and only on numbers (percentages, temperatures, hot table
  values). It never fills a graph, where it would compete with identity.
- **The accent (cyan) means "you can press this"**: key letters in hints, the default button. It
  is not decoration.
- **Gray means secondary**: units, zeros, PID / USER / TIME, separators.

## 2. Separate by tone and space, not lines

- No borders around tiles or sections, no rules between rows. In RGB, areas are panels a tone
  above the background; in ANSI, gutters and blank rows do the same job.
- The one exception: overlays in ANSI keep a border, because without background tones nothing
  else separates them from what's underneath.

## 3. The value is the hero

- In a title, the number is bold and its unit dim (`10.28W`); the label is the subsystem's color.
- Secondary text is dim, so names and values stand forward.
- A section name reads like a name (`Processes`, `Memory`), not an abbreviation.

## 4. A column is one subsystem

- The grid reads top to bottom: load over power (CPU over CPU W, GPU over GPU W), with memory and
  total power in the third column. Stacked graphs (CPU tiers, total power by source) carry a legend
  in their title colors.

## 5. Honest scales, no cut numbers

- Loads scale to 100 %; power graphs scale to a floor (2 W, total 5 W) so idle looks idle.
- A title that doesn't fit drops whole pieces, never part of a number.
- A graph that says nothing (flat memory history) gives its space to something that does (meters).

## 6. Hints where you act

- Actions live next to what they act on: process actions on the section heading while a row is
  selected; filter keys while typing; global keys in the footer.
- One strong fill marks the selection; the default button is the accent fill.

## 7. ANSI first, RGB adds

- ANSI (the terminal's own palette) is the default and must stand on its own in light and dark
  themes. RGB (when the terminal reports truecolor) adds surfaces, smooth gradients and shading,
  never information ANSI lacks.

## 8. Data over chrome

- Space goes to graphs and numbers. Chrome (version, app name, labels repeated elsewhere) is
  minimal and quiet.

## Checks for every change

Capture it with betamax in: dark ANSI, light ANSI, RGB, 800 px wide, and real hardware (process list
hidden in anything public).
