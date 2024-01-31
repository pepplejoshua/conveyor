#include <stdio.h>   // for printf and fprintf
#include <stdlib.h>  // for atoi
#include <pthread.h> // for threads
#include <stdbool.h> // for bool and true and false

typedef struct {
  int* array;
  int size;
} IntArray;

typedef struct {
  int thread_id; // the id of the thread
  IntArray data; // the data to process
  int count; // the number of primes found
  FILE *output_file; // the file to write the primes to
  pthread_mutex_t *mutex; // the mutex to lock when writing to the file
} ThreadData;

bool is_prime(int n) {
  if (n <= 1) return false;
  for (int i = 2; i * i <= n; ++i) {
    if (n % i == 0) return false; 
  }
  return true;
}

IntArray read_file(FILE *file) {
  int num_a, num_b;
  int cap = 10; // initial capacity
  int *numbers = malloc(cap * sizeof(int));
  if (numbers == NULL) {
    perror("Error allocating memory for reading numbers");
    exit(EXIT_FAILURE);
  }

  int size = 0;
  while (fscanf(file, "%d,%d\n", &num_a, &num_b) == 2) {
    // check if we need to resize the array
    if (size >= cap) {
      cap *= 2;
      // realloc returns a pointer to the new array after resizing
      numbers = realloc(numbers, cap * sizeof(int));
      if (numbers == NULL) {
        perror("Error reallocating memory for reading numbers");
        exit(EXIT_FAILURE);
      }
    }

    numbers[size] = num_a;
    size++;
    numbers[size] = num_b;
    size++;
  }

  // resize the array to the correct size
  numbers = realloc(numbers, size * sizeof(int));
  if (numbers == NULL) {
    perror("Error right-sizing memory after reading numbers");
    exit(EXIT_FAILURE);
  }

  IntArray result = {numbers, size};
  return result;
}

void* thread_compute_primes(void *arg) {
  ThreadData *data = (ThreadData *)arg;
  int count = 0;

  int* primes = malloc(data->data.size * sizeof(int));

  for (int i = 0; i < data->data.size; ++i) {
    if (is_prime(data->data.array[i])) {
      primes[count] = data->data.array[i];
      count++;
    }
  }

  for (int i = 0; i < count; ++i) {
    pthread_mutex_lock(data->mutex);
    fprintf(data->output_file, "%d\n", primes[i]);
    pthread_mutex_unlock(data->mutex);
  }

  data->count = count;
  pthread_exit(NULL);
}

int main(int argc, char *argv[]) {
  if (argc != 4) {
    fprintf(stderr, "Usage: %s <filename> <num_threads> <output_filename>\n", argv[0]);
    exit(EXIT_FAILURE);
  }

  char *input = argv[1];
  char *output = argv[3];
  int num_threads = atoi(argv[2]);

  FILE *file = fopen(input, "rb");
  if (file == NULL) {
    perror("Error opening file");
    exit(EXIT_FAILURE);
  }

  FILE *output_file = fopen(output, "w");
  if (output_file == NULL) {
    perror("Error opening file");
    exit(EXIT_FAILURE);
  }

  // read the contents of the file into an array and close it
  IntArray data = read_file(file);

  // print the contents of the array
  // for (int i = 0; i < data.size; ++i) {
  //   printf("%d. %d\n", i + 1, data.array[i]);
  // }

  int slice_size = data.size / num_threads; // size of each slice given to a thread

  // create an array of threads
  pthread_t *threads = malloc(num_threads * sizeof(pthread_t));
  if (threads == NULL) {
    perror("Error allocating memory for threads");
    exit(EXIT_FAILURE);
  }

  // create an array of thread data
  ThreadData *thread_data = malloc(num_threads * sizeof(ThreadData));
  if (thread_data == NULL) {
    perror("Error allocating memory for thread data");
    exit(EXIT_FAILURE);
  }

  // create output file mutex
  pthread_mutex_t mutex;
  pthread_mutex_init(&mutex, NULL);

  for (int i = 0; i < num_threads; ++i) {
    thread_data[i].thread_id = i;
    thread_data[i].data.array = data.array + (i * slice_size);
    thread_data[i].data.size = i == num_threads - 1 ? data.size - (i * slice_size) : slice_size;
    thread_data[i].count = 0;
    thread_data[i].output_file = output_file;
    thread_data[i].mutex = &mutex;

    // create the thread
    pthread_create(&threads[i], NULL, thread_compute_primes, (void *)&thread_data[i]);
  }

  // display thread data info
  // for (int i = 0; i < num_threads; ++i) {
  //   printf("Thread %d:\n", i);
  //   printf("  thread_id: %d\n", thread_data[i].thread_id);
  //   printf("  data: ");
  //   for (int j = 0; j < thread_data[i].data.size; ++j) {
  //     printf("%d ", thread_data[i].data.array[j]);
  //   }
  //   printf("\n");
  //   printf("  count: %d\n", thread_data[i].count);
  //   printf("\n");
  // }

  // wait for the threads to finish
  for (int i = 0; i < num_threads; ++i) {
    pthread_join(threads[i], NULL);
  }

  // display primes found by each thread
  int total_count = 0;
  for (int i = 0; i < num_threads; ++i) {
    printf("Thread %d: %d primes\n", i, thread_data[i].count);
    total_count += thread_data[i].count;
  }

  printf("\nTotal prime numbers found: %d\n", total_count);
  fprintf(output_file, "Total prime numbers found: %d\n", total_count);

  // free memory
  free(data.array);
  free(threads);
  free(thread_data);

  // close files
  fclose(file);
  fclose(output_file);

  // destroy mutex
  pthread_mutex_destroy(&mutex);

  return 0;
}