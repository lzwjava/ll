import random

def quicksort(arr, low, high):
    if low < high:
        pivot = partition(arr, low, high)
        quicksort(arr, low, pivot - 1)
        quicksort(arr, pivot + 1, high)

def partition(arr, low, high):
    pivot = arr[high]
    i = low
    for j in range(low, high):
        if arr[j] <= pivot:
            arr[i], arr[j] = arr[j], arr[i]
            i += 1
    arr[i], arr[high] = arr[high], arr[i]
    return i

import time

nums = [random.randint(0, 9999) for _ in range(10000)]
print("Before:", nums[:10])
start = time.perf_counter()
quicksort(nums, 0, len(nums) - 1)
elapsed = time.perf_counter() - start
print("After: ", nums[:10])
print(f"Sorted 10000 numbers in {elapsed*1000:.3f} ms")
