// Past `thin_from` metres from the eye (`GrassUniform::card_thin_from`: 40 while the cards only cast the cascades'
// shadows, nowhere while they are traced) a tuft stands as a card only by chance, the share kept falling with the
// square of the distance down to `CARD_THIN_LEAST`, each at its own size: every far tuft drawn into four cascades a
// frame made 150 m of grass shadow cost three milliseconds and stutter. A tuft whose chance passes the share shrinks to
// its foot over `CARD_THIN_SOFT` of it past the share, so one leaving or joining as the eye moves never pops, and while
// every tuft is kept every one stands whole. Widened to cover as much as the tufts left out, a far tuft's shadow stood
// two or three times its size and shrank as the eye came nearer.
const CARD_THIN_LEAST: f32 = 0.11;
const CARD_THIN_SOFT: f32 = 0.25;

// The share of the tufts `from_eye` metres from the eye that stand as cards, thinned past `thin_from`.
fn card_kept(from_eye: f32, thin_from: f32) -> f32 {
  let near: f32 = thin_from / max(from_eye, thin_from);

  return max(near * near, CARD_THIN_LEAST);
}

// The share of its size a tuft of chance `chance`, from nothing to one, stands at as a card where `kept` of the tufts
// stand as cards: whole up to the share, none `CARD_THIN_SOFT` of it past.
fn card_thinned(kept: f32, chance: f32) -> f32 {
  return saturate((kept - chance) / (kept * CARD_THIN_SOFT) + 1.0);
}
