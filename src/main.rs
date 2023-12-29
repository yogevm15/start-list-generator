#![feature(int_roundings)]

use std::cmp::max;
use std::collections::VecDeque;
use std::ops::{Add, Div};

use chrono::Duration;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};

#[derive(PartialEq)]
enum Origin {
    Top,
    Bottom,
    Current,
}

type Minutes = isize;
struct Competitor {
    origin: Origin,
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

fn generate_startlist(
    mut windows: Vec<Window>,
    spacing_threshold: Minutes,
    min_spacing: Minutes,
) -> Vec<CompetitorWithOffset> {
    let mut competitors_count: isize = 0;
    let mut entire_duration = 0;

    for window in windows.iter_mut() {
        window
            .competitors
            .make_contiguous()
            .shuffle(&mut thread_rng());
        competitors_count += window.competitors.len() as isize;
        entire_duration += window.duration;
    }
    if competitors_count <= 0 {
        return vec![];
    }

    let entire_spacing = entire_duration.div_ceil(competitors_count);
    let entire_spacing = if entire_spacing <= spacing_threshold {
        // the entire spacing is smaller than spacing_threshold,
        // thus we need don't need to stabilize, just use the entire spacing for all competitors.
        Some(entire_spacing)
    } else {
        stabilize_windows(&mut windows, spacing_threshold);

        None
    };

    let mut competitors = Vec::with_capacity(competitors_count as usize);
    let mut curr_start = 0;
    let mut windows_curr_start = 0;
    for mut window in windows.into_iter() {
        if window.competitors.len() as i32 != 0 {
            for i in 0..window.competitors.len() {
                if window.competitors[0].origin == Origin::Bottom {
                    competitors.push(CompetitorWithOffset {
                        competitor: window.competitors.pop_front().unwrap(),
                        offset: curr_start,
                    });
                    curr_start += spacing_threshold;
                } else {
                    break;
                }
            }

            let mut rev_curr_start = windows_curr_start + window.duration - 1;
            for i in (0..window.competitors.len()).rev() {
                if window.competitors[window.competitors.len() - 1].origin == Origin::Top {
                    competitors.push(CompetitorWithOffset {
                        competitor: window.competitors.pop_back().unwrap(),
                        offset: rev_curr_start,
                    });
                    rev_curr_start -= spacing_threshold;
                } else {
                    break;
                }
            }

            let mut remaining_competitors = window.competitors.len() as isize;

            curr_start -= spacing_threshold;
            if remaining_competitors != 0 {
                let remaining_space = rev_curr_start + spacing_threshold - curr_start;
                let (spacing, mut remainder) = (
                    remaining_space / (remaining_competitors + 1),
                    remaining_space % (remaining_competitors + 1),
                );

                let mut rng = thread_rng();

                for comp in window.competitors {
                    if comp.origin == Origin::Current {
                        if rev_curr_start != (windows_curr_start + window.duration - 1)
                            && rng.gen_bool(remainder as f64 / remaining_competitors as f64)
                        {
                            curr_start += 1;
                            remainder -= 1;
                        }
                        curr_start += spacing;
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
                    curr_start + spacing_threshold,
                    windows_curr_start + window.duration,
                )
            } else {
                windows_curr_start + window.duration - 1 + spacing_threshold
            };
        }
        windows_curr_start += window.duration;
    }
    competitors
}

fn move_to_prev_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Minutes) {
    let mut popped_competitor = windows[i].competitors.pop_front().unwrap();
    popped_competitor.origin = Origin::Top;
    windows[i - 1].competitors.push_back(popped_competitor);
    stabilize_windows(windows, spacing_threshold);
}

fn move_to_next_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Minutes) {
    let mut popped_competitor = windows[i].competitors.pop_back().unwrap();
    popped_competitor.origin = Origin::Bottom;
    windows[i + 1].competitors.push_front(popped_competitor);

    stabilize_windows(windows, spacing_threshold);
}

fn stabilize_windows(windows: &mut Vec<Window>, spacing_threshold: Minutes) {
    let rand_i = thread_rng().gen_range(0..windows.len());
    for i in 0..windows.len() {
        stabilize_window(windows, (i + rand_i) % windows.len(), spacing_threshold);
    }
}

fn calculate_window_space(duration: Minutes, competitors_count: isize) -> Minutes {
    if competitors_count == 0 {
        return duration;
    }

    duration.div(competitors_count)
}

fn calculate_window_space_exact(duration: Minutes, competitors_count: isize) -> f64 {
    if competitors_count == 0 {
        return duration as f64;
    }

    (duration as f64).div(competitors_count as f64)
}

fn stabilize_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Minutes) {
    if calculate_window_space_exact(windows[i].duration, windows[i].competitors.len() as isize)
        < spacing_threshold as f64
    {
        // curr is passed the spacing threshold, thus competitors movement is possible
        let curr_window_space = calculate_window_space_exact(
            windows[i].duration,
            (windows[i].competitors.len()) as isize,
        );
        if i > 0 && i < windows.len() - 1 {
            // Normal window
            let next_window_spacing = calculate_window_space_exact(
                windows[i + 1].duration,
                windows[i + 1].competitors.len() as isize,
            );
            let prev_window_spacing = calculate_window_space_exact(
                windows[i - 1].duration,
                windows[i - 1].competitors.len() as isize,
            );
            if next_window_spacing > curr_window_space && next_window_spacing > prev_window_spacing
            {
                // next is spaced than curr and prev
                move_to_next_window(windows, i, spacing_threshold);
            } else if prev_window_spacing > curr_window_space
                && prev_window_spacing > next_window_spacing
            {
                // prev is spaced than curr and next
                move_to_prev_window(windows, i, spacing_threshold);
            } else if prev_window_spacing > curr_window_space
                && prev_window_spacing == next_window_spacing
            {
                if thread_rng().gen_bool(0.5) {
                    move_to_prev_window(windows, i, spacing_threshold);
                } else {
                    move_to_next_window(windows, i, spacing_threshold);
                }
            }
        } else if i > 0
            && calculate_window_space_exact(
                windows[i - 1].duration,
                windows[i - 1].competitors.len() as isize,
            ) > curr_window_space
        {
            // Last window
            move_to_prev_window(windows, i, spacing_threshold);
        } else if i < windows.len() - 1
            && calculate_window_space_exact(
                windows[i + 1].duration,
                windows[i + 1].competitors.len() as isize,
            ) > curr_window_space
        {
            // First window
            move_to_next_window(windows, i, spacing_threshold);
        }
    }
}

fn main() {
    let spacing_threshold = 3;
    let min_spacing = 2;

    let mut time_windows = vec![
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([
                Competitor {
                    origin: Origin::Current,
                    name: String::from("Levit Kristina"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("טייטר מקסים"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("צינדר אריאל"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("שפר שי"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("UKR Ersteniuk Volodymyr"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("ליזוגוב ויאצ'סלב"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("שחורי דניאל"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("שפר שחר"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("אורי אופיר"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("גמינדר תומר"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("ויינר תומר"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("לדרר חגי"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("מלץ אבי"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("רגבי לירון"),
                },
            ]),
        },
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([
                Competitor {
                    origin: Origin::Current,
                    name: String::from("קלקשטיין בר"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("קלקשטיין יובל"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("שחורי אלון"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("ילין יואב"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("לרמן שגיא"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("לשצ'נקו ניקיטה"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("נוסבוים איתם"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("קלקשטיין נדב"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("רז-רוטשילד דניאל"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("ריינליב דינסטי"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("שפירא אורן"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("אשכנזי אסף"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("גלוזשטיין ולרי"),
                },
            ]),
        },
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([
                Competitor {
                    origin: Origin::Current,
                    name: String::from("מאירקוביץ מארינה"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("מצפון גאיה"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("אילין רומן"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("מאירקוביץ יבגני"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("רון ירדן"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("שפיצר בן ארי"),
                },
                Competitor {
                    origin: Origin::Current,
                    name: String::from("קורוטייב מיכאיל"),
                },
            ]),
        },
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([]),
        },
    ];
    time_windows.reverse();
    let mut result = generate_startlist(time_windows, spacing_threshold, min_spacing);
    let start_time = chrono::naive::NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    result.sort_by(|c1, c2| c1.offset.cmp(&c2.offset));
    for (i, competitor_with_offset) in result.iter().enumerate() {
        println!(
            "[{}] Competitor: {}, time: {}",
            i + 1,
            competitor_with_offset.competitor.name,
            start_time.add(Duration::minutes(competitor_with_offset.offset as i64))
        );
    }
}
