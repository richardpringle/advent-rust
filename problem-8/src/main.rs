use std::{error::Error, fs, string::ToString};

#[derive(Debug)]
struct Layer<'a> {
    data: &'a [usize],
    width: usize,
    height: usize,
}

impl<'a> Layer<'a> {
    fn count_occurrences_of(&self, digit: usize) -> usize {
        self.data.iter().filter(|x| **x == digit).count()
    }
}

fn get_layers(data: &[usize], width: usize, height: usize) -> Vec<Layer> {
    data.chunks(width * height)
        .map(|data| Layer {
            data,
            width,
            height,
        })
        .collect()
}

fn get_image(layers: Vec<Layer>, width: usize) -> String {
    let image = layers[0].data.to_vec();
    let raw_image = layers.into_iter().skip(1).fold(image, |mut image, layer| {
        image.iter_mut().enumerate().for_each(|(i, x)| {
            if *x == 2 {
                *x = layer.data[i];
            }
        });

        image
    });

    raw_image
        .into_iter()
        .map(|x| x.to_string())
        .map(|x| if &x == "1" { "0" } else { " " })
        .collect::<Vec<_>>()
        .chunks(width)
        .map(|chunk| chunk.join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn solve_1(input: &[usize]) -> usize {
    let width = 25;
    let height = 6;
    let layer = get_layers(input, width, height)
        .into_iter()
        .min_by(|a, b| a.count_occurrences_of(0).cmp(&b.count_occurrences_of(0)))
        .expect("No min!?");

    layer.count_occurrences_of(1) * layer.count_occurrences_of(2)
}

fn solve_2(input: &[usize]) -> String {
    let width = 25;
    let height = 6;
    let layers = get_layers(input, width, height);
    get_image(layers, width)
}

fn main() {
    let input = get_input().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    println!("first solution: {:?}", solve_1(&input));
    println!("second solution:");
    print!("{}", solve_2(&input));
    println!();
}

fn get_input() -> Result<Vec<usize>, Box<dyn Error>> {
    Ok(fs::read_to_string("input.txt")?
        .trim()
        .chars()
        .map(|char| char.to_digit(10).unwrap() as usize)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_1_test() {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2];
        let width = 3;
        let height = 2;
        let mut layers = get_layers(&input, width, height).into_iter();

        let layer_1 = layers.next().unwrap();
        let layer_2 = layers.next().unwrap();

        assert_eq!(layer_1.data, &[1, 2, 3, 4, 5, 6]);
        assert_eq!(layer_2.data, &[7, 8, 9, 0, 1, 2]);
    }

    #[test]
    fn solve_2_test() {
        let input = vec![0, 2, 2, 2, 1, 1, 2, 2, 2, 2, 1, 2, 0, 0, 0, 0];
        let width = 2;
        let height = 2;

        let layers = get_layers(&input, width, height);
        let image = get_image(layers, width);

        assert_eq!(image, "  0\n0  ");
    }
}
