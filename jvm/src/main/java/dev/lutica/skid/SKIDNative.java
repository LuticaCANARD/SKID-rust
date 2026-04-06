package dev.lutica.skid;

/**
 * LuticaSKID JNI 바인딩.
 *
 * GPU 가속 이미지 처리 라이브러리의 Java/Kotlin 인터페이스.
 * 핸들(long) 기반으로 네이티브 이미지를 관리하며,
 * 반드시 사용 후 {@link #free(long)}를 호출해야 한다.
 *
 * <pre>{@code
 * long handle = SKIDNative.createFromF32Array(pixels, 1920, 1080);
 * try {
 *     long resized = SKIDNative.resize(handle, 3840, 2160);
 *     try {
 *         float[] data = SKIDNative.getDataAsF32Array(resized);
 *         // ... use data
 *     } finally {
 *         SKIDNative.free(resized);
 *     }
 * } finally {
 *     SKIDNative.free(handle);
 * }
 * }</pre>
 */
public class SKIDNative {

    static {
        System.loadLibrary("skid_rust_backend");
    }

    // ─── 이미지 생명주기 ───

    /**
     * float[] 배열로부터 이미지를 생성한다.
     * 배열 형식: 인터리브 RGBA (R0,G0,B0,A0,R1,G1,B1,A1,...).
     * 각 채널 값은 0.0~1.0 범위.
     *
     * @param data   RGBA 인터리브 float 배열 (길이 = width * height * 4)
     * @param width  이미지 너비
     * @param height 이미지 높이
     * @return 이미지 핸들 (0이면 실패)
     */
    public static native long createFromF32Array(float[] data, int width, int height);

    /**
     * 이미지 핸들을 해제한다. 사용 후 반드시 호출할 것.
     *
     * @param handle 해제할 이미지 핸들
     */
    public static native void free(long handle);

    // ─── 이미지 속성 ───

    /**
     * 이미지 크기를 반환한다.
     * 반환값: 상위 32비트 = width, 하위 32비트 = height.
     *
     * @param handle 이미지 핸들
     * @return 패킹된 크기 (0이면 유효하지 않은 핸들)
     */
    public static native long getSize(long handle);

    /**
     * 패킹된 크기에서 너비를 추출한다.
     */
    public static int getWidth(long handle) {
        return (int) (getSize(handle) >>> 32);
    }

    /**
     * 패킹된 크기에서 높이를 추출한다.
     */
    public static int getHeight(long handle) {
        return (int) (getSize(handle) & 0xFFFFFFFFL);
    }

    // ─── 이미지 데이터 ───

    /**
     * 이미지 데이터를 RGBA 인터리브 float 배열로 반환한다.
     *
     * @param handle 이미지 핸들
     * @return RGBA float 배열, 또는 null (유효하지 않은 핸들)
     */
    public static native float[] getDataAsF32Array(long handle);

    // ─── GPU 이미지 처리 ───

    /**
     * 이미지를 리사이즈한다 (바이리니어 보간, GPU 가속).
     * 원본 이미지는 변경되지 않으며, 새 핸들이 반환된다.
     *
     * @param handle    원본 이미지 핸들
     * @param newWidth  새 너비
     * @param newHeight 새 높이
     * @return 리사이즈된 이미지 핸들 (0이면 실패)
     */
    public static native long resize(long handle, int newWidth, int newHeight);

    /**
     * 높이맵에서 노멀맵을 생성한다 (GPU 가속).
     * 원본 이미지는 변경되지 않으며, 새 핸들이 반환된다.
     *
     * @param handle  원본 높이맵 이미지 핸들
     * @param xFactor X축 기울기 배율 (일반적으로 0.5)
     * @param yFactor Y축 기울기 배율 (일반적으로 0.5)
     * @return 노멀맵 이미지 핸들 (0이면 실패)
     */
    public static native long generateNormalMap(long handle, float xFactor, float yFactor);
}
