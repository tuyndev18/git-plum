/**
 * Bản cài canvas 2D của `GraphRenderer` — checkpoint #2 (ROADMAP), nhánh "dựng
 * canvas trước".
 *
 * **Chỉ vẽ.** Không tính lane, không suy ra liên thông, không đọc
 * `commit.parents` — toàn bộ hình học đến từ `GraphRow` mà Rust đã tính (ràng
 * buộc số 1 của CONTEXT.md). Không gắn event listener nào lên canvas (ràng
 * buộc số 5 — canvas thuần trình bày; bắt sự kiện bấm là việc của `div` hàng
 * trong `CommitList.tsx`).
 */

import {
  colorFor,
  laneX,
  ROW_HEIGHT,
  NODE_RADIUS,
  NODE_STROKE_WIDTH,
  MERGE_NODE_RADIUS,
  NODE_FILL,
  NODE_FILL_VAR,
  NODE_HALO_WIDTH,
  SELECTION_RING,
  EDGE_WIDTH,
} from './geometry'
import type { GraphRenderer, GraphRendererFactory, GraphRenderRow } from './types'
import type { Edge } from '@/lib/ipc'

/** Ngữ cảnh vẽ 2D tối thiểu mà bản cài này cần — cho phép tiêm giả lập ở test. */
export interface DrawingContext2D {
  save(): void
  restore(): void
  scale(x: number, y: number): void
  clearRect(x: number, y: number, w: number, h: number): void
  fillRect(x: number, y: number, w: number, h: number): void
  beginPath(): void
  moveTo(x: number, y: number): void
  lineTo(x: number, y: number): void
  bezierCurveTo(
    cp1x: number,
    cp1y: number,
    cp2x: number,
    cp2y: number,
    x: number,
    y: number,
  ): void
  arc(x: number, y: number, radius: number, startAngle: number, endAngle: number): void
  stroke(): void
  fill(): void
  fillText(text: string, x: number, y: number): void
  setLineDash(segments: number[]): void
  strokeStyle: string
  fillStyle: string
  lineWidth: number
  /** Không bắt buộc: bối cảnh giả ở test có thể bỏ qua, `drawRow` gán phòng thủ. */
  font?: string
}

/** Điểm tiêm dùng riêng cho test — sản xuất thật dùng mặc định (canvas DOM thật). */
export interface CanvasRendererOptions {
  createCanvasElement?: () => HTMLCanvasElement
}

/**
 * Bán kính góc bo khi đường rẽ gập từ ngang sang dọc. Nhỏ so với `LANE_WIDTH`
 * để góc vẫn đọc ra là "gập vuông", chỉ mềm cạnh cho đỡ gắt.
 */
const CORNER_RADIUS = 4

/**
 * Màu tô lòng nút, lấy từ `--graph-node-fill` của host để khớp nền thật ở cả
 * theme sáng và tối. Rơi về `NODE_FILL` khi không có DOM (test) hoặc khi biến
 * chưa được khai.
 */
function readNodeFill(host: HTMLElement): string {
  if (typeof window === 'undefined' || typeof window.getComputedStyle !== 'function') {
    return NODE_FILL
  }
  // `host` là một đối tượng giả ở test (không phải Element thật), và
  // `getComputedStyle` ném `TypeError` cho tham số như vậy. Bọc try/catch thay
  // vì kiểm tra `instanceof Element`: bộ vẽ không được biết gì về hình dạng
  // host ngoài việc nó là chỗ để gắn canvas — đúng tinh thần điểm tiêm
  // `createCanvasElement` đã có.
  try {
    const value = window.getComputedStyle(host).getPropertyValue(NODE_FILL_VAR).trim()
    return value === '' ? NODE_FILL : value
  } catch {
    return NODE_FILL
  }
}

/**
 * Vẽ một đoạn nối giữa hai lane. Dọc nếu cùng lane; **gập vuông góc** nếu khác
 * lane — đi ngang trước rồi gập xuống, với một góc bo nhỏ.
 *
 * Vì sao gập vuông chứ không bezier chéo: ảnh tham chiếu
 * (`docs/screenshots/main-4.png`) cho thấy đường rẽ đi **ngang** sang lane đích
 * rồi **gập xuống** theo góc vuông. Đó là lý do đồ thị ở đó dễ lần theo dù có
 * hàng chục nhánh cạnh nhau: mắt bám được đoạn ngang và đoạn dọc riêng biệt,
 * trong khi nhiều đường bezier chéo cùng đi qua một vùng thì gần như không
 * phân biệt được đường nào là đường nào — đúng hiện tượng người dùng báo "rất
 * khó nhìn". Với `LANE_WIDTH` 14px và `ROW_HEIGHT` 28px, một đường bezier chéo
 * còn có độ dốc tới ~63°, làm nó gần như trùng hướng với đường dọc bên cạnh.
 */
function drawEdge(ctx: DrawingContext2D, edge: Edge, yStart: number, yEnd: number): void {
  drawSegment(ctx, laneX(edge.fromLane), laneX(edge.toLane), yStart, yEnd, colorFor(edge.color))
}

/** Như `drawEdge` nhưng nhận thẳng toạ độ x — dùng cho đoạn nối nội bộ của hàng. */
function drawSegment(
  ctx: DrawingContext2D,
  xStart: number,
  xEnd: number,
  yStart: number,
  yEnd: number,
  color: string,
): void {

  ctx.strokeStyle = color
  // Đường dày 2px: tham chiếu vẽ lane đủ đậm để phân biệt được màu ở tỉ lệ
  // 100%. Đường 1px mặc định bị mảnh và nhoè khi nhiều lane cạnh nhau.
  ctx.lineWidth = EDGE_WIDTH
  ctx.beginPath()
  ctx.moveTo(xStart, yStart)

  if (xStart === xEnd) {
    ctx.lineTo(xEnd, yEnd)
  } else {
    // Gập vuông: ngang tới gần lane đích, bo góc, rồi dọc xuống.
    //
    // Đoạn ngang chạy ở `yStart` (ngang qua tâm hàng nguồn, nơi có nút commit)
    // nên đường rẽ mọc ra đúng từ nút chứ không lửng lơ giữa hai hàng. Góc bo
    // không được lớn hơn nửa khoảng cách còn lại theo cả hai chiều, nếu không
    // nó sẽ vượt qua điểm đến khi hai lane sát nhau hoặc hàng quá thấp.
    const dx = xEnd - xStart
    const dy = yEnd - yStart
    const r = Math.min(CORNER_RADIUS, Math.abs(dx) / 2, Math.abs(dy) / 2)
    const sweep = Math.sign(dx)

    ctx.lineTo(xEnd - sweep * r, yStart)
    // Bo góc bằng bezier ngắn: điểm điều khiển đặt đúng tại góc vuông lý
    // thuyết `(xEnd, yStart)` nên đường cong tiếp tuyến với cả đoạn ngang và
    // đoạn dọc — không có chỗ gãy.
    ctx.bezierCurveTo(xEnd, yStart, xEnd, yStart, xEnd, yStart + r)
    ctx.lineTo(xEnd, yEnd)
  }

  ctx.stroke()
  // Trả `lineWidth` về mặc định: context dùng chung cho cả lượt vẽ, để nguyên
  // 2px sẽ làm mọi nét sau đó (viền nút, vòng chọn) dày theo ngoài ý muốn.
  ctx.lineWidth = 1
}

/** Vẽ toàn bộ nội dung của một hàng: passthrough, outEdges, nút tròn, chỉ báo. */
function drawRow(
  ctx: DrawingContext2D,
  item: GraphRenderRow,
  selectedCommitId: string | null,
  nodeFill: string,
) {
  const { row, y } = item
  const yCenter = y + ROW_HEIGHT / 2
  const yBottom = y + ROW_HEIGHT

  const nodeX = laneX(row.lane)

  // `passthrough` chứa HAI loại cạnh, không phải một:
  //
  //  a) `fromLane === toLane` — nhánh song song đi xuyên qua hàng, mép trên ->
  //     mép dưới. Đây là ca mà bản trước giả định cho *mọi* phần tử.
  //  b) `fromLane !== toLane` — **điểm hợp nhánh** (bước 2 của `lanes.rs`):
  //     một lane khác đang chờ đúng commit này, nên nó chạy xuống rồi rẽ vào
  //     lane của hàng và chết ở đó. Nó KHÔNG được đi tiếp xuống mép dưới —
  //     lane nguồn đã bị giải phóng, dưới hàng này không còn gì ở đó.
  //
  // Vẽ (b) như (a) là lý do đồ thị trông "vỡ": đường hợp nhánh bị kéo thẳng
  // qua mép dưới vào một lane đã chết, tạo ra các đoạn cụt không nối vào đâu.
  for (const edge of row.passthrough) {
    if (edge.fromLane === edge.toLane) {
      drawEdge(ctx, edge, y, yBottom)
    } else {
      // Hợp nhánh: từ mép TRÊN của lane nguồn, rẽ vào TÂM hàng (nơi có nút),
      // đúng như tham chiếu vẽ nhánh con chui vào nút merge.
      drawEdge(ctx, edge, y, yCenter)
    }
  }

  // Đoạn nối TỪ MÉP TRÊN XUỐNG TÂM trên chính lane của hàng.
  //
  // Không backend nào sinh cạnh này: `passthrough` chỉ mô tả các lane KHÁC
  // (`lanes.rs` bước 4 bỏ qua `i == lane`), còn `out_edges` bắt đầu từ tâm đi
  // xuống. Nên nửa TRÊN của ô, ngay trên nút, không ai vẽ — đó là khe hở làm
  // các nút trông như những vòng tròn rời rạc trôi nổi thay vì nằm trên một
  // đường lane liên tục như tham chiếu (`docs/screenshots/main-4.png`).
  //
  // Hàng đầu của một nhánh (không có con nào phía trên trong lane này) vẫn vẽ
  // đoạn này: nó chỉ dài nửa hàng, và nếu nhánh thật sự bắt đầu ở đây thì hàng
  // trên sẽ có cạnh rẽ vào lane này chạm đúng mép chung — hai bên khớp nhau.
  drawSegment(ctx, nodeX, nodeX, y, yCenter, colorFor(row.color))

  // Cạnh nối từ nút hàng này xuống MÉP DƯỚI của chính hàng này — **không** kéo
  // sang tận tâm hàng kế tiếp.
  //
  // Mỗi hàng chỉ được vẽ trong ô của chính nó: nửa dưới do `outEdges` của hàng
  // này vẽ (tâm -> mép dưới), nửa trên của hàng kế tiếp do `passthrough` hoặc
  // `outEdges` của hàng đó vẽ. Hai nửa gặp nhau ở mép chung nên đường liền
  // mạch.
  //
  // Vẽ tới `y + 1.5 * ROW_HEIGHT` (bản trước) làm đường tràn vào ô của hàng
  // dưới và **vẽ đè** lên đường mà hàng đó tự vẽ, ở lane khác nhau — đó là
  // nguyên nhân đồ thị trông chồng chéo và "vỡ" khi có nhánh. Lỗi càng rõ khi
  // virtualizer chỉ dựng các hàng đang thấy: đường tràn vẫn vẽ xuống một hàng
  // có thể chưa được dựng, nên nó không bao giờ khớp với gì cả.
  for (const edge of row.outEdges) {
    drawEdge(ctx, edge, yCenter, yBottom)
  }

  // Nút của chính hàng này — **chấm ĐẶC màu lane**, có quầng nền tách nó khỏi
  // các đường chạy phía sau.
  //
  // Đo pixel tham chiếu (`docs/06-graph-render-model.md` mục 2.3): lòng nút là
  // ĐÚNG màu lane, đặc hoàn toàn — không phải vòng rỗng tô nền như bản trước.
  // Bản trước khoét tâm bằng màu nền nên ở mật độ cao (13 lane, ảnh người dùng
  // gửi) mỗi nút thành một lỗ thủng giữa một rừng sọc, không đọc ra là "điểm"
  // nữa.
  //
  // Quầng nền (`nodeFill`) vẽ trước, rộng hơn chấm 2px: nó cắt mọi đường đi
  // ngang/dọc phía sau đúng quanh nút, nên chấm luôn nổi lên kể cả khi có cạnh
  // merge cùng màu chạy sát bên. Đây là thứ thay cho việc tham chiếu có avatar
  // 22px che hẳn vùng đó.
  const laneColor = colorFor(row.color)
  // Merge commit = có nhiều hơn một cạnh đi ra. Nút merge lớn hơn để mắt nhận
  // ra điểm hợp nhánh khi lần theo lịch sử.
  const isMerge = row.outEdges.length > 1
  const radius = isMerge ? MERGE_NODE_RADIUS : NODE_RADIUS

  ctx.beginPath()
  ctx.arc(nodeX, yCenter, radius + NODE_HALO_WIDTH, 0, Math.PI * 2)
  ctx.fillStyle = nodeFill
  ctx.fill()

  if (row.terminates) {
    // Hàng kết thúc (biên trang/shallow): vòng NÉT ĐỨT rỗng, phân biệt bằng
    // hình dạng chứ không chỉ bằng màu — đây là ca duy nhất còn vẽ vòng rỗng,
    // đúng vì nó phải trông khác hẳn một commit bình thường.
    ctx.beginPath()
    ctx.arc(nodeX, yCenter, radius, 0, Math.PI * 2)
    ctx.strokeStyle = laneColor
    ctx.lineWidth = NODE_STROKE_WIDTH
    ctx.setLineDash([3, 3])
    ctx.stroke()
    ctx.setLineDash([])
    ctx.lineWidth = 1
  } else {
    ctx.beginPath()
    ctx.arc(nodeX, yCenter, radius, 0, Math.PI * 2)
    ctx.fillStyle = laneColor
    ctx.fill()
  }

  // Merge có thêm một lỗ tối ở tâm — dấu hiệu thứ hai ngoài kích thước, để
  // phân biệt được cả khi hai nút cạnh nhau cùng màu.
  if (isMerge && !row.terminates) {
    ctx.beginPath()
    ctx.arc(nodeX, yCenter, radius / 2.5, 0, Math.PI * 2)
    ctx.fillStyle = nodeFill
    ctx.fill()
  }

  if (row.commitId === selectedCommitId) {
    ctx.beginPath()
    ctx.arc(nodeX, yCenter, radius + 3, 0, Math.PI * 2)
    ctx.strokeStyle = SELECTION_RING
    ctx.lineWidth = 2
    ctx.stroke()
    ctx.lineWidth = 1
  }

  if (row.truncatedParents > 0) {
    // Phông phải đặt rõ: mặc định của canvas là `10px sans-serif`, nhỏ và khác
    // hẳn phần chữ còn lại của khung — chỉ báo `+N` trở nên gần như không đọc
    // được. 10px bold khớp cỡ chữ của `.commit-header` trong `app.css`.
    ctx.font = '600 10px system-ui, sans-serif'
    ctx.fillStyle = laneColor
    ctx.fillText(`+${row.truncatedParents}`, nodeX + radius + 3, yCenter + 3)
  }
}

/** `GraphRendererFactory` cài bằng canvas 2D. Tham số thứ hai chỉ dùng ở test. */
export const createCanvasRenderer: (
  host: HTMLElement,
  options?: CanvasRendererOptions,
) => GraphRenderer = (host, options) => {
  const canvas =
    options?.createCanvasElement?.() ??
    (typeof document !== 'undefined' ? document.createElement('canvas') : undefined)

  if (!canvas) {
    throw new Error('createCanvasRenderer: không có document và không có createCanvasElement')
  }

  canvas.style.position = canvas.style.position || 'absolute'
  host.appendChild(canvas)

  const ctx = canvas.getContext('2d') as unknown as DrawingContext2D | null
  if (!ctx) {
    throw new Error('createCanvasRenderer: getContext("2d") trả null')
  }

  let cssWidth = 0
  let cssHeight = 0

  const renderer: GraphRenderer = {
    resize(width, height, dpr) {
      cssWidth = width
      cssHeight = height
      canvas.width = Math.round(width * dpr)
      canvas.height = Math.round(height * dpr)
      canvas.style.width = `${width}px`
      canvas.style.height = `${height}px`
      // `scale` phải gọi đúng MỘT lần sau mỗi resize — gọi tích luỹ nhiều lần
      // sẽ khiến hình vẽ ra sai tỉ lệ ở lần resize thứ hai.
      ctx.scale(dpr, dpr)
    },

    draw(rows, selectedCommitId) {
      ctx.clearRect(0, 0, cssWidth, cssHeight)

      // KHÔNG tô nền cho cả cột đồ thị.
      //
      // Bản trước `fillRect` toàn cột bằng `NODE_FILL` — một khối đục phủ lên
      // dải nền xen kẽ của `.commit-row:nth-child(odd)` (canvas nằm TRÊN các
      // hàng về thứ tự vẽ, xem `.graph-canvas` trong `app.css`). Kết quả là cột
      // đồ thị thành một mảng phẳng cắt rời khỏi cột chữ, trong khi tham chiếu
      // (`docs/screenshots/main-4.png`) cho dải nền chạy LIỀN từ cột nhãn qua
      // cột đồ thị sang cột thông điệp — đó là thứ giúp mắt lần theo một hàng
      // ngang qua đồ thị. Canvas trong suốt để dải nền của hàng hiện xuyên qua;
      // riêng lòng nút vẫn được khoét bằng `NODE_FILL` ở `drawRow`.
      // Đọc lại mỗi lượt vẽ chứ không cache: người dùng đổi theme hệ điều hành
      // thì `--graph-node-fill` đổi theo `prefers-color-scheme` mà không có
      // event nào cho canvas biết. `getComputedStyle` trên một phần tử là phép
      // đọc rẻ, và một lượt vẽ chỉ gọi đúng một lần cho cả trăm hàng.
      const nodeFill = readNodeFill(host)

      for (const item of rows) {
        drawRow(ctx, item, selectedCommitId, nodeFill)
      }
    },

    dispose() {
      host.removeChild(canvas)
    },
  }

  return renderer
}

// Khẳng định kiểu tĩnh: `createCanvasRenderer` (khi gọi với một tham số) khớp
// `GraphRendererFactory` — điểm nối của checkpoint #2.
const _typeCheck: GraphRendererFactory = (h) => createCanvasRenderer(h)
void _typeCheck
