#![feature(iter_map_windows)]

use std::cmp::max;
use std::collections::VecDeque;
use std::ops::{Add, Div};

use chrono::Duration;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};

type Minutes = isize;
struct Competitor {
    origin: isize, // positive->top, negative->bottom, zero->current
    name: String,
}

struct CompetitorWithOffset {
    competitor: Competitor,
    offset: Minutes,
}

struct Window {
    duration: Minutes,
    competitors: VecDeque<Competitor>,
}

impl Window {
    fn calculate_spacing(&self) -> f64 {
        if self.competitors.len() == 0 {
            return self.duration as f64;
        }

        (self.duration as f64).div(self.competitors.len() as f64)
    }
}

fn generate_startlist(
    mut windows: Vec<Window>,
    spacing_threshold: Minutes,
    min_spacing: Minutes,
) -> Vec<CompetitorWithOffset> {
    let mut competitors_count: isize = 0;

    for window in windows.iter_mut() {
        window
            .competitors
            .make_contiguous()
            .shuffle(&mut thread_rng());
        competitors_count += window.competitors.len() as isize;
    }
    if competitors_count <= 0 {
        return vec![];
    }

    stabilize_windows(&mut windows, spacing_threshold);
    smart_offset_assignments(windows, spacing_threshold, min_spacing, competitors_count)
}

fn smart_offset_assignments(
    windows: Vec<Window>,
    spacing_threshold: Minutes,
    min_spacing: Minutes,
    competitors_count: isize,
) -> Vec<CompetitorWithOffset> {
    let mut competitors = Vec::with_capacity(competitors_count as usize);
    let mut curr_start = 0;
    let mut windows_curr_start = 0;
    for mut window in windows.into_iter() {
        if window.competitors.len() as i32 != 0 {
            let mut has_bottom = false;
            while window.competitors.len() > 0 {
                if window.competitors[0].origin < 0 {
                    has_bottom = true;
                    competitors.push(CompetitorWithOffset {
                        competitor: window.competitors.pop_front().unwrap(),
                        offset: curr_start,
                    });
                    curr_start += spacing_threshold;
                } else {
                    break;
                }
            }
            if has_bottom {
                curr_start -= spacing_threshold;
            }

            let mut rev_curr_start = windows_curr_start + window.duration - 1;
            let mut top_competitors = Vec::with_capacity(window.competitors.len());
            while window.competitors.len() > 0 {
                if window.competitors[window.competitors.len() - 1].origin > 0 {
                    top_competitors.push(CompetitorWithOffset {
                        competitor: window.competitors.pop_back().unwrap(),
                        offset: rev_curr_start,
                    });
                    rev_curr_start -= spacing_threshold;
                } else {
                    break;
                }
            }

            let mut remaining_competitors = window.competitors.len() as isize;

            if remaining_competitors != 0 {
                let remaining_space = rev_curr_start - curr_start;
                let (spacing, mut remainder) = (
                    remaining_space / (remaining_competitors),
                    remaining_space % (remaining_competitors),
                );

                let mut rng = thread_rng();
                let mut first_in_window = !has_bottom;
                for comp in window.competitors {
                    if comp.origin == 0 {
                        if !first_in_window {
                            if spacing >= min_spacing {
                                if rng.gen_bool(remainder as f64 / remaining_competitors as f64) {
                                    curr_start += 1;
                                    remainder -= 1;
                                }
                                curr_start += spacing;
                            } else {
                                curr_start += min_spacing;
                            }
                        } else {
                            first_in_window = false;
                        }
                        competitors.push(CompetitorWithOffset {
                            competitor: comp,
                            offset: curr_start,
                        });
                        remaining_competitors -= 1;
                    } else {
                        break;
                    }
                }
            }

            curr_start = if rev_curr_start == (windows_curr_start + window.duration - 1) {
                max(
                    curr_start + min_spacing,
                    windows_curr_start + window.duration,
                )
            } else {
                windows_curr_start + window.duration - 1 + spacing_threshold
            };

            competitors.extend(top_competitors.into_iter().rev());
        }
        windows_curr_start += window.duration;
    }
    competitors
}

fn move_to_prev_window(windows: &mut Vec<Window>, i: usize) {
    let mut popped_competitor = windows[i].competitors.pop_front().unwrap();
    popped_competitor.origin += 1;
    windows[i - 1].competitors.push_back(popped_competitor);
}

fn move_to_next_window(windows: &mut Vec<Window>, i: usize) {
    let mut popped_competitor = windows[i].competitors.pop_back().unwrap();
    popped_competitor.origin -= 1;
    windows[i + 1].competitors.push_front(popped_competitor);
}

fn calculate_max_diff(windows: &Vec<Window>) -> f64 {
    let iter = windows.iter().map(|w| w.calculate_spacing());
    iter.clone()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        - iter.min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

fn stabilize_windows(windows: &mut Vec<Window>, spacing_threshold: Minutes) {
    if windows.len() < 2 {
        return;
    }

    let mut last_movement = None::<((usize, f64), (usize, f64), f64)>;
    let mut last_max_diff = f64::MAX;
    loop {
        let diffs = (0..windows.len())
            .map(|i| (i, windows[i].calculate_spacing()))
            .map_windows(|[s1, s2]| (s1.clone(), s2.clone(), s1.1 - s2.1))
            .filter(|(s1, s2, _)| {
                s1.1 <= spacing_threshold as f64 || s2.1 <= spacing_threshold as f64
            });
        let curr_movement = diffs.max_by(|d1, d2| d1.2.abs().partial_cmp(&d2.2.abs()).unwrap());
        if curr_movement.is_none() {
            break;
        }
        let curr_movement = curr_movement.unwrap();
        let curr_max_diff = calculate_max_diff(windows);
        if (curr_max_diff > last_max_diff)
            || last_movement.is_some_and(|(_, _, last_diff)| last_diff.abs() == curr_max_diff)
        {
            match last_movement {
                Some(((_, _), (src, _), diff)) if diff < 0.0 => {
                    move_to_prev_window(windows, src);
                }
                Some(((src, _), (_, _), diff)) if diff > 0.0 => {
                    move_to_next_window(windows, src);
                }
                _ => {
                    unreachable!();
                }
            }
            break;
        }
        match curr_movement {
            ((src, _), (_, _), diff) if diff < 0.0 => {
                move_to_next_window(windows, src);
            }
            ((_, _), (src, _), diff) if diff > 0.0 => {
                move_to_prev_window(windows, src);
            }
            _ => {
                break;
            }
        }
        last_movement.replace(curr_movement);
        last_max_diff = curr_max_diff;
    }
}

fn main() {
    let spacing_threshold = 3;
    let min_spacing = 2;

    let mut time_windows = vec![];

    time_windows.push(Window {
        duration: 30,
        competitors: {
            let mut competitors = VecDeque::new();
            for i in 0..2 {
                competitors.push_front(Competitor {
                    name: format!("1 Competitor {}", i),
                    origin: 0,
                })
            }
            competitors
        },
    });
    time_windows.push(Window {
        duration: 30,
        competitors: {
            let mut competitors = VecDeque::new();
            for i in 0..15 {
                competitors.push_front(Competitor {
                    name: format!("2 Competitor {}", i),
                    origin: 0,
                })
            }
            competitors
        },
    });
    time_windows.push(Window {
        duration: 30,
        competitors: {
            let mut competitors = VecDeque::new();
            for i in 0..4 {
                competitors.push_front(Competitor {
                    name: format!("3 Competitor {}", i),
                    origin: 0,
                })
            }
            competitors
        },
    });
    let result = generate_startlist(time_windows, spacing_threshold, min_spacing);
    let start_time = chrono::naive::NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    for (i, competitor_with_offset) in result.iter().enumerate() {
        println!(
            "[{}] Competitor: {}, time: {}",
            i + 1,
            competitor_with_offset.competitor.name,
            start_time.add(Duration::minutes(competitor_with_offset.offset as i64))
        );
    }
}
