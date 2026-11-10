// The paint figure's starting puzzle: 6×6, one star, made by choosing the stars first
// (star-battle's `Attempts::new(6, 1, 1)`, unique on its 16th attempt).
const regions = "444100440000450000450000553320533330";

export const PAINT_START = {
  size: 6,
  stars: 1,
  regions: [...regions].map(Number),
};
