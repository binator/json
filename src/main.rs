use std::collections::HashMap;

fn main() {
  let log2: String = std::fs::read_to_string("log3").unwrap();

  let mut hist = HashMap::new();
  for line in log2.lines() {
    let mut line = line.split(' ');
    let name = line.next().unwrap();
    let time = line.next().unwrap();
    let time: f64 = time
      .parse()
      .map_err(|e| {
        print!("{}", time);
        e
      })
      .unwrap();
    let unit = line.next().unwrap();
    let time = match unit {
      "µs" => time,
      "ms" => time * 1_000.,
      "ns" => time / 1_000.,
      _ => panic!(),
    };

    hist.entry(name).or_insert(Vec::new()).push(time);
  }

  let mut hist: Vec<_> = hist.into_iter().collect();
  hist.sort_by_key(|x| x.0);

  for (name, times) in hist.iter_mut() {
    times.sort_by(|a, b| f64::partial_cmp(a, b).unwrap());

    let mean = times.iter().copied().sum::<f64>() / times.len() as f64;
    let max = times.iter().copied().reduce(|a, b| a.max(b)).unwrap();
    let min = times.iter().copied().reduce(|a, b| a.min(b)).unwrap();

    let average = times
      .iter()
      .skip(times.len() / 20)
      .take(times.len() * 9 / 10)
      .copied()
      .sum::<f64>()
      / times.len() as f64;
    println!(
      "{}: average {:.1} mean {:.1} max {} min {} n {}",
      name,
      average,
      mean,
      max,
      min,
      times.len()
    );
  }
}
