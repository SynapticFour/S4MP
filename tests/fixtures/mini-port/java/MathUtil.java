package demo;

/** Cross-file callee for mini-port e2e. */
public final class MathUtil {
    private MathUtil() {}

    public static int scale(int x) {
        return x * 2;
    }
}
