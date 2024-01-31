#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{sync::{Arc, Mutex}, thread};
use std::io::Write;

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
  // for (index, number) in numbers.lock().unwrap().iter().enumerate() {
  //   println!("{index}. {number}", index=index+1);
  // }

  // create a vector to hold thread data
  let mut thread_data: Vec<Arc<Mutex<ThreadData>>> = Vec::new();
  let slice_size = num_len / num_threads;
  for i in 0..num_threads {
    let n_thread_data = ThreadData {
      thread_id: i,
      data: Arc::clone(&numbers),
      primes_count: 0,
      start: i * slice_size,
      // if there are 3 threads and 5 numbers
      // slice_size = 5 / 3 = 1
      // thread 0: 0th number
      // thread 1: 1st number
      // thread 2: slice_size = 5 - (2 * 1) = 3
      // thread 2: 2nd, 3rd, 4th number
      size: if i == num_threads - 1 {
        num_len - (i * slice_size)
      } else {
        slice_size
      },
      output_file_mutex: Arc::clone(&output_file),
    };

    thread_data.push(Arc::new(Mutex::new(n_thread_data)));
  }

  // dispatch threads
  let mut handles = vec![];
  let thread_data = Arc::new(thread_data);
  for data in thread_data.iter() {
    // make arc copies of the thread data before moving them into the thread
    // this is because the thread will take ownership of the data and we want to keep a reference to it
    let data = Arc::clone(&data);
    let handle = thread::spawn(move || {
      let mut primes: Vec<usize> = Vec::new();
      let mut data = data.lock().unwrap();
      {
        let numbers = data.data.lock().unwrap();
        for i in data.start..(data.start + data.size) {
          if is_prime(numbers[i]) {
            primes.push(numbers[i]);
          }
        }
      }
      
      data.primes_count = primes.len();
      let mut output_file = data.output_file_mutex.lock().unwrap();
      for prime in primes {
        writeln!(output_file, "{}", prime).unwrap();
      }
    });
    handles.push(handle);
  }

  // wait for threads to finish
  for handle in handles {
    handle.join().unwrap();
  }

  // print the number of primes found by each thread
  let mut total_primes = 0;
  for data in thread_data.iter() {
    let data = data.lock().unwrap();
    println!("Thread {}: {} primes", data.thread_id, data.primes_count);
    total_primes += data.primes_count;
  }

  println!("\nTotal prime numbers found: {}", total_primes);
  writeln!(output_file.lock().unwrap(), "Total prime numbers found: {}", total_primes).unwrap();
}