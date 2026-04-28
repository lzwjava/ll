import java.util.Random;

public class Quicksort {
    static void quicksort(int[] arr, int low, int high) {
        if (low < high) {
            int pivot = partition(arr, low, high);
            quicksort(arr, low, pivot - 1);
            quicksort(arr, pivot + 1, high);
        }
    }

    static int partition(int[] arr, int low, int high) {
        int pivot = arr[high];
        int i = low;
        for (int j = low; j < high; j++) {
            if (arr[j] <= pivot) {
                int tmp = arr[i];
                arr[i] = arr[j];
                arr[j] = tmp;
                i++;
            }
        }
        int tmp = arr[i];
        arr[i] = arr[high];
        arr[high] = tmp;
        return i;
    }

    public static void main(String[] args) {
        Random rng = new Random(42);
        int[] nums = new int[10000];
        for (int i = 0; i < nums.length; i++) {
            nums[i] = rng.nextInt(10000);
        }
        System.out.print("Before: ");
        for (int i = 0; i < 10; i++) { System.out.print(nums[i] + " "); }
        System.out.println();

        long start = System.nanoTime();
        quicksort(nums, 0, nums.length - 1);
        long elapsed = System.nanoTime() - start;

        System.out.print("After:  ");
        for (int i = 0; i < 10; i++) { System.out.print(nums[i] + " "); }
        System.out.println();
        System.out.printf("Sorted 10000 numbers in %.3f ms%n", elapsed / 1_000_000.0);
    }
}
