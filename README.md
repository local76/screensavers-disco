# disco

> A high-energy rave with confetti, pulsing stars, and your live OS logo in the middle of the celebration.

Neon confetti explodes across the screen while background stars pulse and react. A disco ball effect and the live OS logo sit in the center. The whole scene gets more intense when your system is under load.

## Visual elements

- **Confetti**. Colorful particles with limited lifetime, using neon colors blended with your accent.
- **Stars**. Background points that get excited and pulse.
- **Disco ball** (optional). A central pulsing light source.
- **Live logo**. The OS name + kernel rendered with a rave-rainbow cycling color effect.

## Dynamic / live behavior

- **Live logo**. Dynamic via `get_system_info()`.
- **System load reactions**. Higher CPU/memory usage increases confetti count, star excitation rate, and overall energy. The party gets wilder when your machine is working hard.
- **Host bias**. Subtle per-computer differences in pulse timing and color bias.
- **Accent + audio**. Colors heavily use your system accent. If audio visualization is active it can further drive excitement.

## Configuration (registry)

Under `HKEY_CURRENT_USER\Software\local76\disco`:

- `ConfettiDensity`: 0 = light, 1 = normal, 2 = heavy.
- `DiscoBall`: 0 / 1 — whether the central disco ball effect is enabled.

## Notes

- One of the most fun and high-contrast scenes.
- Works especially well with a strong system accent color.
- The live data integration means even the party has a little bit of your actual OS identity in it.

Part of the [screensavers](https://github.com/local76/screensavers) collection. See the root README for installation.
