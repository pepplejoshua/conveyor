#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{sync::{Arc, Mutex}, thread};

struct ThreadData {
  thread_id: usize,
  data: Arc<Mutex<Vec<usize>>>,
  start: usize,
  size: usize,
  primes_count: usize,
  output_file_mutex: Arc<Mutex<std::fs::File>>,
}


fn read_file(filename: &str) -> Vec<usize> {
  let file = std::fs::read_to_string(filename).expect("Error opening input file");
  let mut numbers: Vec<usize> = Vec::new();
  for line in file.lines() {
    // each line has DIGIT,DIGIT format
    let mut parts = line.split(",");
    let first = parts.next().unwrap().parse::<usize>().unwrap();
    let second = parts.next().unwrap().parse::<usize>().unwrap();
    numbers.push(first);
    numbers.push(second);
  }
  numbers
}

fn is_prime(n: usize) -> bool {
  if n <= 1 {
    return false;
  }
  let mut i = 2;
  loop {
    if i * i > n {
      break;
    }
    if n % i == 0 {
      return false;
    }
    i += 1;
  }
  true
}

fn main() {
  // read args. expecting 3 args
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 4 {
    eprintln!("Usage: {} <filename> <num_threads> <output_filename>", args[0]);
    std::process::exit(1);
  }

  // parse args
  let filename = &args[1];
  let num_threads = args[2].parse::<usize>().unwrap();
  let output_filename = &args[3];
  
  // open output file. it can be created if it doesn't exist
  let output_file = std::fs::OpenOptions::new()
    .write(true)
    .create(true)
    .open(output_filename)
    .expect("Error opening output file");

  let output_file = Arc::new(Mutex::new(output_file));


  // read the contents of the file into an array and close it
  let numbers = read_file(filename);
  let num_len = numbers.len();
  let numbers = Arc::new(Mutex::new(numbers));

  // print the contents of the array
  for (index, number) in numbers.lock().unwrap().iter().enumerate() {
    println!("{index}. {number}", index=index+1);
  }

  // create a vector to hold thread data
  let mut thread_data: Vec<ThreadData> = Vec::new();
  let slice_size = num_len / num_threads;
  for i in 0..num_threads {
    thread_data.push(ThreadData {
      thread_id: i,
      data: Arc::clone(&numbers),
      primes_count: 0,
      start: i * slice_size,
      size: if i == num_threads - 1 {
        num_len - (i * slice_size)
      } else {
        slice_size
      },
      output_file_mutex: Arc::clone(&output_file),
    });
  }
}