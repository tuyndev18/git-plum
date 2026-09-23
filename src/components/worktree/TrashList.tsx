/**
 * Danh sách "Vừa huỷ gần đây" — WORK-07.
 *
 * # 🔴 Khoảng trống đã biết, và vì sao component này NÓI RA thay vì giấu
 *
 * 05-03-SUMMARY và doc comment của `ipc.listTrash` ghi rõ: phía Rust dựng danh sách
 * bằng `git for-each-ref`, và `for-each-ref` **không giữ được** `paths`, `luc`, `nhan`
 * — ba trường đó chỉ tồn tại lúc `luu_truoc_khi_huy` chạy. Backend trả
 * `paths: []`, `luc: 0`, `nhan: ''` cho **mọi** mục.
 *
 * Wave 4 ghi điều đó vào doc comment **cốt để wave này không dựng một danh sách trống
 * rồi mới hiểu tại sao**. Nên đây là chỗ quyết định phải làm gì với nó, và quyết định
 * đó có hai nửa:
 *
 * ## Nửa một: `luc` lấy lại được từ `refName`, KHÔNG cần chỗ lưu phụ
 *
 * Tên ref do `commands/trash.rs` dựng là `refs/git-plum-trash/{epoch_ms}-{n}` — mốc
 * mili giây **nằm ngay trong tên**. `for-each-ref` trả tên ref, nên thời điểm huỷ
 * **không bao giờ mất**; nó chỉ chưa được đọc ra.
 *
 * Đây là phần khoảng trống **đóng được ở giao diện**, và đóng ở đây là đúng chỗ:
 * `refName` đã đi qua IPC rồi, thêm một chỗ lưu phụ ở Rust chỉ để chuyển cùng con số
 * đó lần thứ hai là việc thừa. Có test ghim (`data-ms`), và nó khẳng định **không**
 * hiện 1970 — tức không đọc `luc: 0`.
 *
 * ## Nửa hai: `paths` và `nhan` thì KHÔNG lấy lại được, và danh sách nói thẳng
 *
 * Tên tệp thật sự mất: không có gì trong `refName` hay `objectId` mang nó. Lấy lại
 * được chỉ bằng một chỗ lưu phụ ở Rust (ghi chú ref, hoặc một tệp chỉ mục) — **việc
 * có thật, chưa làm**, ghi là nợ ở SUMMARY.
 *
 * 🔴 Cho tới lúc đó, danh sách **nói ra** rằng tên tệp chưa có, và hiện `objectId`
 * rút gọn để người dùng còn phân biệt được hai mục. Một danh sách ba dòng trắng trơn
 * là một danh sách người dùng **không dám bấm**: họ không biết mục nào chứa gì, và
 * nút ở đây **ghi đè nội dung tệp của họ**. Thà nói thẳng "chưa ghi lại tên tệp" còn
 * hơn hiện khoảng trắng rồi để họ đoán.
 *
 * Có test ghim **cả hai** chiều: mục không có `nhan` phải hiện câu giải thích, và mục
 * **có** `nhan` phải hiện `nhan`. Thiếu chiều thứ hai, một cài đặt bỏ qua `nhan` hoàn
 * toàn sẽ qua được cổng, và khoảng trống không bao giờ đóng được dù backend đã sửa.
 *
 * # 🔴 Nợ T-05-11 — `restore_trash` KHÔNG tự lưu trước khi ghi đè
 *
 * Ghi lại từ 05-03-SUMMARY: `khoi_phuc` ghi đè thẳng lên tệp đang có. Khôi phục đè
 * lên một thay đổi người dùng vừa gõ sẽ **mất** thay đổi đó. Chưa cài; vẫn là nợ.
 *
 * Component này **không** tự vá nó bằng một hộp thoại xác nhận: một `confirm()` ở
 * giao diện là lớp phòng thủ sai chỗ (T-05-13 nói rõ mọi đường huỷ phải tự lưu ở
 * **Rust**, nơi `BienNhan` không dựng được ngoài crate). Thêm nó ở đây sẽ tạo cảm
 * giác an toàn cho một đường **vẫn** mất dữ liệu khi gọi từ chỗ khác.
 *
 * # ⚠️ Bố cục CHƯA KIỂM CHỨNG
 *
 * happy-dom không tính CSS layout và không có cuộn thật (`CONTEXT.md` 4.5). "Danh sách
 * không tràn hoặc cắt chữ khi có nhiều mục" và "tìm thấy được" là **có mã, chưa kiểm**
 * cho tới khi có người xem trên Chromium thật — checkpoint 05-05 bước 8 và câu bố cục
 * thứ ba. Khuôn doc comment chép từ `ChangeList.tsx`.
 *
 * Đây không phải từ chối lấy lệ: lỗi `marginTop`→`paddingTop` của Phase 4 đi qua **cả
 * 666 test**, và cả **năm** lỗi hiển thị Phase 3 thoát toàn bộ test tự động.
 */

import { useCallback, useEffect, useState } from 'react'

import { describeError, ipc, type MucThungRac } from '@/lib/ipc'
import { useStatusStore } from '@/stores/statusStore'

/**
 * Mốc mili giây đọc từ tên ref, hoặc `null` khi tên không theo khuôn.
 *
 * Khuôn do `commands/trash.rs` dựng: `refs/git-plum-trash/{epoch_ms}-{n}`.
 *
 * 🔴 `null` khi không đọc được, **không** phải `0`. `0` là một mốc **hợp lệ**
 * (1970-01-01) và nó sẽ được định dạng thành một ngày trông như thật — tức một lỗi
 * phân tích trở thành một thông tin sai mà người dùng tin. Cùng lý do `ahead: null`
 * khác `ahead: 0` ở `BranchInfo`.
 */
export function mocTuRefName(refName: string): number | null {
  const duoi = refName.slice(refName.lastIndexOf('/') + 1)
  const cat = duoi.indexOf('-')
  const phan = cat === -1 ? duoi : duoi.slice(0, cat)
  if (phan === '' || !/^\d+$/.test(phan)) return null
  const n = Number(phan)
  return Number.isSafeInteger(n) ? n : null
}

/**
 * Thời điểm huỷ, định dạng theo locale người dùng.
 *
 * `Intl.DateTimeFormat` chứ không tự ghép chuỗi: CLAUDE.md loại `chrono` khỏi phía
 * Rust **chính vì** `Intl` xử lý locale và múi giờ đúng và miễn phí. Không khẳng định
 * chuỗi kết quả trong test — nó đổi theo máy chạy test; test khẳng định thứ phân biệt
 * được (`data-ms`, và "không phải 1970").
 */
function dinhDangLuc(ms: number): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(ms))
}

interface Props {
  /** Không truyền → không render gì. Khuôn `ChangeList`: bịa khoá `''` ghi đè store repo khác. */
  repoId?: string
}

export function TrashList({ repoId }: Props) {
  const [muc, setMuc] = useState<MucThungRac[] | null>(null)
  const [loi, setLoi] = useState<string | null>(null)

  const nap = useCallback(async (id: string) => {
    try {
      const ds = await ipc.listTrash(id)
      setMuc(ds)
      setLoi(null)
    } catch (e) {
      setLoi(describeError(e))
      // 🔴 KHÔNG `setMuc([])` ở nhánh lỗi. Đặt danh sách rỗng khi lời gọi **hỏng**
      // làm giao diện nói "chưa huỷ gì" — một câu **sai** mà người dùng tin, và họ
      // sẽ kết luận bản lưu của mình không tồn tại. Giữ danh sách cũ; băng lỗi nói
      // ra rằng lần đọc này thất bại. Cùng quyết định với `statusStore.ghi`.
    }
  }, [])

  useEffect(() => {
    if (repoId) void nap(repoId)
  }, [repoId, nap])

  const khoiPhuc = useCallback(
    async (id: string, m: MucThungRac) => {
      try {
        const status = await ipc.restoreTrash(id, m.refName, m.paths)
        // Ràng buộc 2.5: ghi `RepoStatus` **từ giá trị trả về**, KHÔNG gọi `getStatus`.
        // Cùng khuôn `hunkStore.ghiKhoi` và đột biến M2 của 04-02. Có test ghim.
        useStatusStore.getState().applyExternal(id, status)
        setLoi(null)
        await nap(id)
      } catch (e) {
        setLoi(describeError(e))
      }
    },
    [nap],
  )

  if (!repoId) return null

  /*
   * Sắp **mới nhất trước**, ở giao diện.
   *
   * Phía Rust đã `--sort=-refname` và tên ref rộng cố định 13 chữ số, nên nó **cũng**
   * đúng thứ tự. Sắp lại ở đây không phải vì nghi ngờ backend — mà vì thứ tự là một
   * **yêu cầu của giao diện này** (mục vừa huỷ là mục người dùng đang tìm), và một
   * yêu cầu của giao diện phải có cổng ở giao diện. Backend đổi cách sắp mai này thì
   * đây vẫn đúng, và test đỏ sẽ chỉ vào đúng chỗ.
   *
   * `[...muc]` — `sort` sửa mảng **tại chỗ**, và mảng này đến từ state.
   *
   * Mục không đọc được mốc xuống cuối (`-1`), không lên đầu: một tên ref hỏng không
   * được chiếm chỗ của mục vừa huỷ thật.
   */
  const daSap =
    muc === null
      ? null
      : [...muc].sort((a, b) => (mocTuRefName(b.refName) ?? -1) - (mocTuRefName(a.refName) ?? -1))

  return (
    <section className="trash-list" data-testid="trash-list">
      {loi && (
        /*
         * 🔴 `role="alert"` — khuôn M26 của wave 4, và cùng lý do.
         *
         * Băng này là thứ **duy nhất** nói cho người dùng biết lần khôi phục của họ
         * KHÔNG có hiệu lực. Không có `role="alert"`, trình đọc màn hình không đọc
         * nó ra: họ bấm "Khôi phục", không nghe gì, và tin rằng nội dung đã về.
         *
         * Phần tử này **chỉ** tồn tại ở trạng thái lỗi. Gắn `role` lên vỏ
         * `trash-list` — có mặt ở **mọi** trạng thái — là đúng lỗi #6, và có test
         * phủ định ghim (`KHÔNG lỗi → KHÔNG có role="alert" nào`).
         */
        <div className="trash-error" data-testid="trash-error" role="alert">
          {loi}
        </div>
      )}

      {daSap === null ? (
        <p className="trash-loading" data-testid="trash-loading">
          Đang đọc danh sách vừa huỷ…
        </p>
      ) : daSap.length === 0 ? (
        /*
         * 🔴 Danh sách rỗng KHÔNG hiện tiêu đề — khuôn "nhóm rỗng không hiện tiêu đề"
         * của `ChangeList` (đột biến M8 của 04-02). Một tiêu đề "Vừa huỷ gần đây"
         * trên một danh sách rỗng là hai dòng nói "không có gì" một lần.
         */
        <p className="trash-empty" data-testid="trash-empty">
          Chưa huỷ bỏ gì gần đây. Mọi lần huỷ đều được tự lưu vào đây trước.
        </p>
      ) : (
        <>
          <h3 className="trash-heading" data-testid="trash-heading">
            Vừa huỷ gần đây <span className="trash-count">({daSap.length})</span>
          </h3>
          <div className="trash-rows">
            {daSap.map((m) => {
              const ms = mocTuRefName(m.refName)
              return (
                <div
                  key={m.refName}
                  className="trash-row"
                  data-testid="trash-row"
                  // `data-ref` là thứ test thứ tự đọc. Truy vấn theo chữ hiện ra sẽ
                  // không phân biệt được ba mục trống giống hệt nhau.
                  data-ref={m.refName}
                >
                  <span className="trash-nhan" data-testid="trash-nhan">
                    {m.nhan !== ''
                      ? m.nhan
                      : /*
                         * Xem doc comment đầu tệp, nửa hai. Câu này là một **cổng**,
                         * không phải một lời xin lỗi: có test đỏ nếu ai gỡ nó đi.
                         */
                        'Bản lưu này chưa ghi lại tên tệp — danh sách dựng từ tên ref.'}
                  </span>

                  <span className="trash-meta">
                    <code className="trash-oid">{m.objectId.slice(0, 8)}</code>
                    {m.laBlob ? ' · tệp chưa theo dõi' : ' · thay đổi đã theo dõi'}
                  </span>

                  {ms === null ? (
                    // Không đọc được mốc → nói ra. Hiện một ngày bịa còn tệ hơn.
                    <span className="trash-time" data-testid="trash-time" data-ms="">
                      không đọc được thời điểm
                    </span>
                  ) : (
                    <span
                      className="trash-time"
                      data-testid="trash-time"
                      data-ms={String(ms)}
                      // `title` mang dạng máy đọc được, cho người muốn số chính xác.
                      title={new Date(ms).toISOString()}
                    >
                      {dinhDangLuc(ms)}
                    </span>
                  )}

                  <button
                    type="button"
                    className="trash-restore"
                    data-testid="trash-restore"
                    // Tên khả truy cập mang **mã object**, không chỉ chữ "Khôi phục":
                    // ba nút cùng nhãn là ba nút trình đọc màn hình đọc giống hệt
                    // nhau, và người dùng bàn phím không biết mình đang ở nút nào.
                    aria-label={`Khôi phục bản lưu ${m.objectId.slice(0, 8)}`}
                    onClick={() => {
                      void khoiPhuc(repoId, m)
                    }}
                  >
                    Khôi phục
                  </button>
                </div>
              )
            })}
          </div>
        </>
      )}
    </section>
  )
}
