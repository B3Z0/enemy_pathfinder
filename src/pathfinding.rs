use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::map::{MAP_H, MAP_W, blocked_for_agent, is_wall};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Node {
    f: i32,
    g: i32,
    x: usize,
    y: usize,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .cmp(&self.f)
            .then_with(|| other.g.cmp(&self.g))
            .then_with(|| other.y.cmp(&self.y))
            .then_with(|| other.x.cmp(&self.x))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn manhattan(a: (usize, usize), b: (usize, usize)) -> i32 {
    let dx = if a.0 > b.0 { a.0 - b.0 } else { b.0 - a.0 };
    let dy = if a.1 > b.1 { a.1 - b.1 } else { b.1 - a.1 };
    (dx + dy) as i32
}

fn neighbors4(x: usize, y: usize) -> [(i32, i32); 4] {
    [
        (x as i32 + 1, y as i32),
        (x as i32 - 1, y as i32),
        (x as i32, y as i32 + 1),
        (x as i32, y as i32 - 1),
    ]
}

pub fn astar(start: (usize, usize), goal: (usize, usize)) -> Vec<(usize, usize)> {
    if is_wall(start.0, start.1) || is_wall(goal.0, goal.1) {
        return vec![];
    }
    if start == goal {
        return vec![start];
    }

    let mut open = BinaryHeap::<Node>::new();
    let inf: i32 = i32::MAX / 4;

    let mut g_score = [[inf; MAP_W]; MAP_H];
    let mut came_from = [[None::<(usize, usize)>; MAP_W]; MAP_H];
    let mut closed = [[false; MAP_W]; MAP_H];

    g_score[start.1][start.0] = 0;

    open.push(Node {
        x: start.0,
        y: start.1,
        g: 0,
        f: manhattan(start, goal),
    });

    while let Some(current) = open.pop() {
        let cx = current.x;
        let cy = current.y;

        if closed[cy][cx] {
            continue;
        }
        closed[cy][cx] = true;

        if (cx, cy) == goal {
            let mut path = vec![(cx, cy)];
            let mut cur = (cx, cy);
            while cur != start {
                if let Some(prev) = came_from[cur.1][cur.0] {
                    cur = prev;
                    path.push(cur);
                } else {
                    return vec![];
                }
            }
            path.reverse();
            return path;
        }

        let current_g = g_score[cy][cx];

        for (nx, ny) in neighbors4(cx, cy) {
            if nx < 0 || ny < 0 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            if nx >= MAP_W || ny >= MAP_H {
                continue;
            }

            if blocked_for_agent(nx, ny) {
                continue;
            }
            if closed[ny][nx] {
                continue;
            }

            let tentative_g = current_g + 1;
            if tentative_g < g_score[ny][nx] {
                came_from[ny][nx] = Some((cx, cy));
                g_score[ny][nx] = tentative_g;

                let h = manhattan((nx, ny), goal);
                open.push(Node {
                    x: nx,
                    y: ny,
                    g: tentative_g,
                    f: tentative_g + h,
                });
            }
        }
    }

    vec![]
}
