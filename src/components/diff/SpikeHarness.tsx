/**
 * Khung đo hai đường A/B của checkpoint #3 — plan 03-01. **Spike, không phải UI.**
 *
 * Xoá cùng lúc với `diffSpike.ts` khi Task 3 chốt xong đường đi. Đây **không** phải
 * trình xem diff của Phase 3; 03-04 viết cái đó.
 *
 * # Hai cổng, mỗi cổng một việc khác nhau
 *
 * 1. `isPerfEnabled()` ở `App.tsx` quyết định có render component này hay không.
 * 2. `import()` động ở đây quyết định gói CodeMirror có vào bundle đường chính hay
 *    không. **Đây không phải tối ưu vặt**: nếu spike làm phình bundle khởi động thì
 *    nó đã phá đúng thứ nó đi đo (khởi động nhanh là Core Value).
 *
 * Nghĩa là mô-đun này **không** được nhập tĩnh `diffSpike.ts`. Mọi thứ liên quan
 * CodeMirror đi qua `await import('@/lib/diffSpike')` bên trong hàm xử lý.
 */

import { useRef, useState } from 'react'

import { describeError, ipc, type SpikeBlobPair } from '@/lib/ipc'

/** Kết quả một lần chạy đủ hai đường. */
interface KetQua {
  a: number[]
  b: number[]
  pair: SpikeBlobPair
}

/** Kết quả kiểm word-level của đường A. */
interface KetQuaWord {
  changedText: number
  changedLine: number
  htmlDongSua: string
}

/** Làm tròn tới 0.1ms — đủ chính xác để chép vào tài liệu, không giả vờ chính xác hơn. */
function ms(n: number): string {
  return `${n.toFixed(1)}ms`
}

export function SpikeHarness({ repoId }: { repoId: string }) {
  const [commitId, setCommitId] = useState('')
  const [path, setPath] = useState('')
  const [dangChay, setDangChay] = useState(false)
  const [loi, setLoi] = useState<string | null>(null)
  const [ketQua, setKetQua] = useState<KetQua | null>(null)
  const [word, setWord] = useState<KetQuaWord | null>(null)

  // Vùng dựng view. Để ngoài luồng hiển thị chính và có kích thước thật — một phần
  // tử cao 0px sẽ khiến CodeMirror chỉ vẽ một dòng và phép đo mất nghĩa.
  const sanDo = useRef<HTMLDivElement>(null)

  const chay = async () => {
    setLoi(null)
    setKetQua(null)
    setDangChay(true)
    try {
      const parent = sanDo.current
      if (!parent) throw new Error('chưa có vùng dựng view')

      // Lấy dữ liệu MỘT lần cho cả hai đường — đó là điều kiện để phép so sánh có
      // nghĩa (xem chú thích đầu `diffSpike.ts`).
      const pair = await ipc.spikeBlobPair(repoId, commitId.trim(), path.trim())

      // `import()` động: gói CodeMirror chỉ nạp khi thật sự bấm chạy.
      const { measurePathA, measurePathB } = await import('@/lib/diffSpike')

      const a = await measurePathA(pair, parent)
      const b = await measurePathB(pair, parent)

      setKetQua({ a, b, pair })
    } catch (e) {
      setLoi(describeError(e))
    } finally {
      setDangChay(false)
    }
  }

  const chayWordLevel = async () => {
    setLoi(null)
    setWord(null)
    setDangChay(true)
    try {
      const parent = sanDo.current
      if (!parent) throw new Error('chưa có vùng dựng view')

      const { kiemWordLevel } = await import('@/lib/diffSpike')
      setWord(await kiemWordLevel(parent))
    } catch (e) {
      setLoi(describeError(e))
    } finally {
      setDangChay(false)
    }
  }

  return (
    <section className="spike-harness" data-testid="spike-harness">
      <h3>Spike đo diff (checkpoint #3)</h3>

      <div className="spike-inputs">
        <label>
          commitId
          <input
            value={commitId}
            onChange={(e) => setCommitId(e.target.value)}
            placeholder="sha đầy đủ hoặc ngắn"
            data-testid="spike-commit"
          />
        </label>
        <label>
          path
          <input
            value={path}
            onChange={(e) => setPath(e.target.value)}
            placeholder="yarn.lock"
            data-testid="spike-path"
          />
        </label>
        <button onClick={() => void chay()} disabled={dangChay} data-testid="spike-run">
          {dangChay ? 'Đang đo…' : 'Đo hai đường'}
        </button>
        <button
          onClick={() => void chayWordLevel()}
          disabled={dangChay}
          data-testid="spike-word"
        >
          Kiểm word-level
        </button>
      </div>

      {loi && (
        <pre className="spike-error" role="alert">
          {loi}
        </pre>
      )}

      {ketQua && (
        <div className="spike-results">
          {/* Bảng chép được: sáu số ms cộng dữ liệu đầu vào. Một con số ms không
              kèm kích thước đầu vào thì không so lại được ở lần đo sau. */}
          <table data-testid="spike-table">
            <thead>
              <tr>
                <th>Đường</th>
                <th>Lần 1</th>
                <th>Lần 2</th>
                <th>Lần 3</th>
                <th>Tốt nhất</th>
                <th>Tệ nhất</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>A — @codemirror/merge tự tính diff</td>
                {ketQua.a.map((n, i) => (
                  <td key={i}>{ms(n)}</td>
                ))}
                <td>{ms(Math.min(...ketQua.a))}</td>
                <td>{ms(Math.max(...ketQua.a))}</td>
              </tr>
              <tr>
                <td>B — decoration từ git diff</td>
                {ketQua.b.map((n, i) => (
                  <td key={i}>{ms(n)}</td>
                ))}
                <td>{ms(Math.min(...ketQua.b))}</td>
                <td>{ms(Math.max(...ketQua.b))}</td>
              </tr>
            </tbody>
          </table>

          <dl className="spike-meta">
            <dt>rustMs</dt>
            <dd data-testid="spike-rustms">{ketQua.pair.rustMs}</dd>
            <dt>oldLines</dt>
            <dd>{ketQua.pair.oldLines}</dd>
            <dt>newLines</dt>
            <dd>{ketQua.pair.newLines}</dd>
            <dt>oldBytes</dt>
            <dd>{ketQua.pair.oldBytes}</dd>
            <dt>newBytes</dt>
            <dd>{ketQua.pair.newBytes}</dd>
            <dt>patchBytes</dt>
            <dd>{ketQua.pair.patchBytes}</dd>
          </dl>

          {/* Dạng một dòng để chép thẳng vào tài liệu mà không phải gõ lại bảng. */}
          <pre className="spike-copy" data-testid="spike-copy">
            {[
              `A: ${ketQua.a.map((n) => n.toFixed(1)).join(' / ')}`,
              `B: ${ketQua.b.map((n) => n.toFixed(1)).join(' / ')}`,
              `rustMs=${ketQua.pair.rustMs}`,
              `oldLines=${ketQua.pair.oldLines}`,
              `newLines=${ketQua.pair.newLines}`,
              `patchBytes=${ketQua.pair.patchBytes}`,
            ].join('  |  ')}
          </pre>
        </div>
      )}

      {word && (
        <div className="spike-word" data-testid="spike-word-result">
          <p>
            <code>.cm-changedText</code> (tô riêng chữ trong dòng):{' '}
            <strong data-testid="spike-changed-text">{word.changedText}</strong>
            {' — '}
            <code>.cm-changedLine</code> (tô cả dòng):{' '}
            <strong data-testid="spike-changed-line">{word.changedLine}</strong>
          </p>
          <p>
            {word.changedText > 0
              ? 'CÓ word-level: @codemirror/merge tô riêng phần chữ thay đổi trong dòng.'
              : 'KHÔNG có word-level trong dòng: chỉ tô cả dòng.'}
          </p>
          <pre className="spike-word-html">{word.htmlDongSua}</pre>
        </div>
      )}

      {/* Chiều cao thật, nếu không CodeMirror chỉ vẽ một dòng và số đo mất nghĩa. */}
      <div ref={sanDo} className="spike-stage" data-testid="spike-stage" />
    </section>
  )
}
