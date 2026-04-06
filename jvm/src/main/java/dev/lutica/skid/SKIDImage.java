package dev.lutica.skid;

import java.lang.ref.Cleaner;

/**
 * SKIDImage의 자원 안전 래퍼.
 *
 * AutoCloseable을 구현하여 try-with-resources 패턴 사용 가능.
 * try-with-resources를 사용하지 못한 경우, {@link java.lang.ref.Cleaner}가
 * GC 시점에 네이티브 핸들을 해제하는 안전장치 역할을 한다.
 *
 * <pre>{@code
 * try (SKIDImage img = SKIDImage.fromPixels(pixels, 1920, 1080)) {
 *     try (SKIDImage resized = img.resize(3840, 2160)) {
 *         float[] data = resized.getData();
 *     }
 * }
 * }</pre>
 */
public class SKIDImage implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    private static class NativeHandleCleaner implements Runnable {
        private long handle;

        NativeHandleCleaner(long handle) {
            this.handle = handle;
        }

        @Override
        public void run() {
            if (handle != 0) {
                SKIDNative.free(handle);
                handle = 0;
            }
        }
    }

    private SKIDImage(long handle) {
        if (handle == 0) {
            throw new IllegalStateException("Failed to create SKIDImage (handle=0)");
        }
        this.handle = handle;
        this.cleanable = CLEANER.register(this, new NativeHandleCleaner(handle));
    }

    /**
     * RGBA 인터리브 float 배열로부터 이미지를 생성한다.
     */
    public static SKIDImage fromPixels(float[] data, int width, int height) {
        return new SKIDImage(SKIDNative.createFromF32Array(data, width, height));
    }

    /**
     * 이미지 너비를 반환한다.
     */
    public int getWidth() {
        ensureValid();
        return SKIDNative.getWidth(handle);
    }

    /**
     * 이미지 높이를 반환한다.
     */
    public int getHeight() {
        ensureValid();
        return SKIDNative.getHeight(handle);
    }

    /**
     * 이미지 데이터를 RGBA 인터리브 float 배열로 반환한다.
     */
    public float[] getData() {
        ensureValid();
        return SKIDNative.getDataAsF32Array(handle);
    }

    /**
     * 이미지를 리사이즈한다 (GPU 가속).
     * 원본은 변경되지 않으며, 새 SKIDImage가 반환된다.
     */
    public SKIDImage resize(int newWidth, int newHeight) {
        ensureValid();
        return new SKIDImage(SKIDNative.resize(handle, newWidth, newHeight));
    }

    /**
     * 높이맵에서 노멀맵을 생성한다 (GPU 가속).
     */
    public SKIDImage generateNormalMap(float xFactor, float yFactor) {
        ensureValid();
        return new SKIDImage(SKIDNative.generateNormalMap(handle, xFactor, yFactor));
    }

    /**
     * 네이티브 핸들을 반환한다 (고급 사용 시).
     */
    public long getHandle() {
        ensureValid();
        return handle;
    }

    private void ensureValid() {
        if (handle == 0) {
            throw new IllegalStateException("SKIDImage has been closed");
        }
    }

    @Override
    public void close() {
        if (handle != 0) {
            SKIDNative.free(handle);
            handle = 0;
        }
        cleanable.clean();
    }
}
