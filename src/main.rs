fn quicksort(arr: &mut Vec<i32>, low: usize, high: usize) {
    if low < high {
        let pivot = partition(arr, low, high);
        if pivot > 0 {
            quicksort(arr, low, pivot - 1);
        }
        quicksort(arr, pivot + 1, high);
    }
}

fn partition(arr: &mut Vec<i32>, low: usize, high: usize) -> usize {
    let pivot = arr[high];
    let mut i = low;
    for j in low..high {
        if arr[j] <= pivot {
            arr.swap(i, j);
            i = i + 1;
        }
    }
    arr.swap(i, high);
    i
}

fn main() {
    let mut nums: Vec<i32> = (0..10000).map(|_| rand_num()).collect();
    println!("Before: {:?}", &nums[..10]);
    let start = std::time::Instant::now();
    let len = nums.len();
    quicksort(&mut nums, 0, len - 1);
    let elapsed = start.elapsed();
    println!("After:  {:?}", &nums[..10]);
    println!("Sorted 10000 numbers in {:?}", elapsed);
}

fn rand_num() -> i32 {
    static mut SEED: u64 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((SEED >> 33) as i32).abs() % 1000
    }
}
