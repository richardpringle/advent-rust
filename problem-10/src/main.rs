use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    collections::{HashSet, VecDeque},
    error::Error,
    fs,
    hash::Hash,
    iter::Iterator,
};

#[derive(Clone, Copy, Debug)]
struct Point(isize, isize, Space);

impl Point {
    fn norm_squared(&self) -> isize {
        self.0.pow(2) + self.1.pow(2)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
enum Quadrant {
    One,
    Two,
    Three,
    Four,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
struct Slope {
    slope: (isize, isize),
    quadrant: Quadrant,
}

impl Slope {
    fn new(origin: &Point, point: &Point) -> Self {
        let slope = get_slope(*origin, *point);

        let quadrant = match slope {
            (rise, 0) => match rise {
                rise if rise.is_negative() => Quadrant::One,
                _ => Quadrant::Three,
            },
            (0, run) => match run {
                run if run.is_positive() => Quadrant::Two,
                _ => Quadrant::Four,
            },
            (rise, run) if run.is_positive() && rise.is_negative() => Quadrant::One,
            (rise, run) if run.is_positive() && rise.is_positive() => Quadrant::Two,
            (rise, run) if run.is_negative() && rise.is_positive() => Quadrant::Three,
            (rise, run) if run.is_negative() && rise.is_negative() => Quadrant::Four,
            _ => panic!("Could not determine quadrant for: {:?}", slope),
        };

        Slope { slope, quadrant }
    }

    // not a real angle, just used for comparison
    fn angle(&self) -> f32 {
        match self.slope {
            (rise, 0) if rise.is_positive() => std::f32::INFINITY,
            (rise, 0) if rise.is_negative() => std::f32::NEG_INFINITY,
            (rise, run) => rise as f32 / run as f32,
        }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.norm_squared().partial_cmp(&other.norm_squared())
    }
}

impl PartialOrd for Slope {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.quadrant.cmp(&other.quadrant) {
            Ordering::Equal => self.angle().partial_cmp(&other.angle()),
            x => Some(x),
        }
    }
}

impl Ord for Slope {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Space {
    Empty,
    Asteroid,
}

use Space::*;

impl Space {
    fn parse(char: char) -> Self {
        match char {
            '#' => Asteroid,
            _ => Empty,
        }
    }
}

fn get_bounds(map: &[Point]) -> (isize, isize) {
    map.iter().fold((0, 0), |(row, col), point| {
        let row = if point.0 > row { point.0 } else { row };
        let col = if point.1 > col { point.1 } else { col };
        (row, col)
    })
}

fn index_map(map: &[Point]) -> Vec<Vec<Space>> {
    let (last_row, last_col) = get_bounds(map);
    let indexed_map = vec![vec![Empty; last_col as usize + 1]; last_row as usize + 1];

    map.iter()
        .fold(indexed_map, |mut indexed_map, Point(row, col, space)| {
            indexed_map[*row as usize][*col as usize] = *space;
            indexed_map
        })
}

fn get_asteroids(map: &[Point]) -> Vec<&Point> {
    map.iter()
        .filter(|Point(_, _, space)| *space == Asteroid)
        .collect()
}

fn has_clear_path(map: &[Vec<Space>], a: Point, b: Point) -> bool {
    let (a, b) = order(a, b);
    let (rise, run) = get_slope(a, b);
    let (mut x, mut y) = (a.0 + run, a.1 + rise);

    while Point(x, y, Empty) < b {
        if map[x as usize][y as usize] == Asteroid {
            return false;
        }
        y += rise;
        x += run;
    }

    true
}

fn destroy_asteroid(
    map: &mut [Vec<Space>],
    origin: &Point,
    slope: &Slope,
) -> Option<(isize, isize)> {
    let (rise, run) = slope.slope;
    let (mut x, mut y) = (origin.0 + run, origin.1 + rise);

    while x < map[0].len() as isize && y < map[0].len() as isize {
        let loc = &mut map[x as usize][y as usize];

        if *loc == Asteroid {
            *loc = Empty;
            return Some((x, y));
        }

        y += rise;
        x += run;
    }

    None
}

fn get_slope(a: Point, b: Point) -> (isize, isize) {
    if a.0 == b.0 {
        return (if a.1 > b.1 { -1 } else { 1 }, 0);
    }

    if a.1 == b.1 {
        return (0, if a.0 > b.0 { -1 } else { 1 });
    }

    let (rise, run) = (b.1 - a.1, b.0 - a.0);
    let gcd = gcd(rise, run);
    (rise / gcd, run / gcd)
}

fn count_asteroids_in_sight(asteroids: &[&Point], asteroid: &Point, map: &[Vec<Space>]) -> usize {
    asteroids
        .iter()
        .filter(|&&point| point != asteroid)
        .filter(|&&&other| has_clear_path(&map, *asteroid, other))
        .count()
}

#[derive(Debug)]
struct Spinner(VecDeque<Slope>);

impl Iterator for Spinner {
    type Item = Slope;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.pop_front();
        self.0.push_back(next.unwrap());
        next
    }
}

fn get_divisors_iter(x: isize) -> impl DoubleEndedIterator<Item = isize> {
    (1..=x).filter(move |i| x % i == 0)
}

fn order<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a > b {
        (b, a)
    } else {
        (a, b)
    }
}

fn gcd(a: isize, b: isize) -> isize {
    let (a, b) = order(a, b);
    get_divisors_iter(a.abs())
        .rev()
        .skip_while(|divisor| b % divisor != 0)
        .take(1)
        .next()
        .unwrap()
}

fn full_dedup<T: Eq + Hash + Copy>(vec: Vec<T>) -> Vec<T> {
    let mut set = HashSet::new();
    vec.into_iter().filter(|x| set.insert(*x)).collect()
}

fn solve_1(map: &[Point]) -> usize {
    let asteroids = get_asteroids(map);
    let map = index_map(map);

    asteroids
        .iter()
        .map(|asteroid| count_asteroids_in_sight(&asteroids, asteroid, &map))
        .max()
        .unwrap()
}

fn solve_2(map: &[Point]) -> isize {
    let asteroids = get_asteroids(map);
    let mut map = index_map(map);

    let (start, _) = asteroids
        .iter()
        .map(|asteroid| {
            (
                asteroid,
                count_asteroids_in_sight(&asteroids, asteroid, &map),
            )
        })
        .max_by(|(_, a), (_, b)| a.cmp(b))
        .unwrap();

    let mut slopes: Vec<_> = asteroids
        .iter()
        .map(|asteroid| Slope::new(start, asteroid))
        .collect();

    slopes.sort();

    let mut slopes = Spinner(full_dedup(slopes).into_iter().collect());

    let mut first = slopes.next().unwrap();

    while first.slope != (-1, 0) {
        first = slopes.next().unwrap()
    }

    Some(first)
        .into_iter()
        .chain(slopes)
        .map(|point| destroy_asteroid(&mut map, start, &point))
        .filter(|result| result.is_some())
        .take(200)
        .last()
        .flatten()
        .map(|point| 100 * point.0 + point.1)
        .expect("should have found an answer!")
}

fn main() {
    let input = get_input().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    let input = parse_input(&input);

    println!("first solution: {:?}", solve_1(&input));
    println!("second solution: {:?}", solve_2(&input));
}

fn parse_input(input: &[String]) -> Vec<Point> {
    input
        .iter()
        .enumerate()
        .flat_map(|(col, raw)| {
            raw.chars()
                .enumerate()
                .map(move |(row, char)| Point(row as isize, col as isize, Space::parse(char)))
        })
        .collect()
}

fn get_input() -> Result<Vec<String>, Box<dyn Error>> {
    Ok(fs::read_to_string("input.txt")?
        .trim()
        .lines()
        .map(|str| str.to_string())
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn get_bounds_from_map() {
        let max_row = 5;
        let max_col = 4;
        let input: Vec<_> = (0..=max_row)
            .flat_map(|row| (0..=max_col).map(move |col| Point(row, col, Space::Empty)))
            .collect();

        let (row_bound, col_bound) = get_bounds(&input);

        assert_eq!(row_bound, max_row);
        assert_eq!(col_bound, max_col);
    }

    #[test]
    fn adjacent_clear_path() {
        let map = vec![vec![Asteroid, Empty], vec![Empty, Asteroid]];

        let a = Point(0, 0, Asteroid);
        let b = Point(1, 1, Asteroid);

        assert_eq!(has_clear_path(&map, a, b), true);
    }

    #[test]
    fn blocked_and_free() {
        let map = vec![
            vec![Asteroid, Empty, Empty],
            vec![Empty, Asteroid, Empty],
            vec![Empty, Asteroid, Asteroid],
        ];

        let a = Point(0, 0, Asteroid);
        let b = Point(2, 2, Asteroid);
        let c = Point(2, 1, Asteroid);

        assert_eq!(has_clear_path(&map, a, b), false);
        assert_eq!(has_clear_path(&map, a, c), true);
    }

    #[test]
    fn same_x() {
        let map = parse_input(&[
            ".#..#".to_string(),
            ".....".to_string(),
            "#####".to_string(),
            "....#".to_string(),
            "...##".to_string(),
        ]);
        let asteroid = Point(4, 0, Asteroid);
        let map = index_map(&map);

        assert_eq!(has_clear_path(&map, asteroid, Point(4, 4, Asteroid)), false);
    }

    #[test]
    fn create_slopes() {
        let start = Point(1, 1, Empty);
        let one = Point(2, 0, Empty);
        let two = Point(2, 2, Empty);
        let three = Point(0, 2, Empty);
        let four = Point(0, 0, Empty);

        let slope_one = Slope::new(&start, &one);

        assert_eq!(slope_one.quadrant, Quadrant::One);
        assert_eq!(slope_one.slope, (-1, 1));

        let slope_two = Slope::new(&start, &two);

        assert_eq!(slope_two.quadrant, Quadrant::Two);
        assert_eq!(slope_two.slope, (1, 1));

        let slope_three = Slope::new(&start, &three);

        assert_eq!(slope_three.quadrant, Quadrant::Three);
        assert_eq!(slope_three.slope, (1, -1));

        let slope_four = Slope::new(&start, &four);

        assert_eq!(slope_four.quadrant, Quadrant::Four);
        assert_eq!(slope_four.slope, (-1, -1));

        assert_eq!(slope_one.cmp(&slope_two), Ordering::Less);
        assert_eq!(slope_two.cmp(&slope_three), Ordering::Less);
        assert_eq!(slope_three.cmp(&slope_four), Ordering::Less);
    }

    #[test]
    fn compare_slopes() {
        let start = Point(4, 4, Empty);
        let a = Slope::new(&start, &Point(5, 0, Empty));
        let b = Slope::new(&start, &Point(5, 1, Empty));
        let c = Slope::new(&start, &Point(5, 2, Empty));

        assert_eq!(a.cmp(&b), Ordering::Less);
        assert_eq!(b.cmp(&c), Ordering::Less);

        let up = Slope::new(&start, &Point(4, 3, Empty));
        let right = Slope::new(&start, &Point(5, 4, Empty));
        let down = Slope::new(&start, &Point(4, 5, Empty));
        let left = Slope::new(&start, &Point(3, 4, Empty));

        assert_eq!(up.slope, (-1, 0));
        assert_eq!(up.quadrant, Quadrant::One);
        assert_eq!(right.slope, (0, 1));
        assert_eq!(right.quadrant, Quadrant::Two);
        assert_eq!(down.slope, (1, 0));
        assert_eq!(down.quadrant, Quadrant::Three);
        assert_eq!(left.slope, (0, -1));
        assert_eq!(left.quadrant, Quadrant::Four);

        assert_eq!(up.cmp(&right), Ordering::Less);
        assert_eq!(right.cmp(&down), Ordering::Less);
        assert_eq!(down.cmp(&left), Ordering::Less);
    }
}
