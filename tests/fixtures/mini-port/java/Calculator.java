package demo;

/** Tiny fixture for S4MP e2e — not production code. */
public class Calculator {
    public int add(int a, int b) {
        return helper(a) + MathUtil.scale(b);
    }

    public int multiply(int a, int b) {
        return a * b;
    }

    private int helper(int x) {
        return x;
    }
}
