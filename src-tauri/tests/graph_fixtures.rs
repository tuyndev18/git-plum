//! Snapshot gán lane trên **repo git thật** — chín hình dạng của plan 02-01.
//!
//! # Vì sao snapshot chứ không phải assertion viết tay
//!
//! Hình dạng đồ thị của repo mẫu `wide` có 73 hàng × (lane, màu, danh sách cạnh) —
//! hàng trăm con số. Viết tay thì không ai soát nổi và không ai sửa nổi khi thuật toán
//! đổi có chủ ý. Còn `cargo insta diff` thì đọc được ngay: một thay đổi ở bước 3 hiện
//! ra thành mấy dòng lệch, và người sửa quyết định đó là ý mình hay là lỗi.
//!
//! # Nhưng snapshot một mình thì không đủ — T-02-10
//!
//! `cargo insta accept` chấp nhận **mọi** thay đổi, kể cả một hành vi sai. Một snapshot
//! vô tình sai vẫn "đỗ" mãi sau đó. Vì vậy tệp này có **năm assertion cứng** nằm ngoài
//! snapshot, phát biểu những bất biến mà không lần `accept` nào ghi đè được:
//! `rows.len() == commits.len()` và thứ tự hàng, quan hệ `color == lane % LANE_COLORS`,
//! giới hạn lane của cạnh rẽ nhánh, sổ sách cha (vẽ được + cắt bớt), và `terminates`
//! chỉ bật khi thật sự có cha ngoài tập.
//!
//! CI chạy với `INSTA_UPDATE=no` nên snapshot lệch làm đỏ chứ không tự ghi lại.

use git_plum_lib::domain::Commit;
use git_plum_lib::git::parsers::log::{parse_log, LOG_ARGS, LOG_FORMAT};
use git_plum_lib::git::GitCommand;
use git_plum_lib::graph::{assign, GraphRow, LANE_COLORS, MAX_VISIBLE_LANES};
use git_plum_lib::testing;

/// Chiếu `Vec<GraphRow>` thành dạng văn bản gọn, **thay mã commit bằng chỉ số hàng**.
///
/// # Vì sao phải chiếu
///
/// Mã commit đầy đủ 40 ký tự làm snapshot vừa không đọc được vừa dễ vỡ: script fixture
/// đổi một byte nội dung tệp là mọi mã đổi theo, và snapshot đỏ toàn bộ dù **hình dạng
/// đồ thị không đổi gì**. Thay mã bằng chỉ số hàng khiến snapshot nói đúng điều ta
/// muốn ghim — hình dạng — và độc lập với nội dung commit.
///
/// Mỗi hàng một dòng:
/// `r{index} lane={lane} color={color} pass=[{from}->{to}, …] out=[…] trunc={n} term={bool}`
fn chieu_gon(rows: &[GraphRow]) -> String {
    let mut out = String::new();
    for (i, r) in rows.iter().enumerate() {
        let pass: Vec<String> = r
            .passthrough
            .iter()
            .map(|e| format!("{}->{}", e.from_lane, e.to_lane))
            .collect();
        let outs: Vec<String> = r
            .out_edges
            .iter()
            .map(|e| format!("{}->{}", e.from_lane, e.to_lane))
            .collect();
        out.push_str(&format!(
            "r{} lane={} color={} pass=[{}] out=[{}] trunc={} term={}\n",
            i,
            r.lane,
            r.color,
            pass.join(", "),
            outs.join(", "),
            r.truncated_parents,
            r.terminates
        ));
    }
    out
}

/// Chạy `git log --all --date-order` trên một repo mẫu rồi phân tích.
///
/// Dùng [`LOG_ARGS`] và [`LOG_FORMAT`] của plan 02-02, **không** viết lại chuỗi định
/// dạng: `--date-order` là bất biến mà [`assign`] dựa vào, và một call site tự gõ lại
/// args sẽ lặng lẽ bỏ nó.
fn doc_commits(repo: &std::path::Path) -> Vec<Commit> {
    let out = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("không dựng được tokio runtime")
        .block_on(async {
            GitCommand::new(repo)
                .args(LOG_ARGS)
                .arg(LOG_FORMAT)
                .run()
                .await
        })
        .expect("không chạy được git log trên repo mẫu");

    assert!(out.is_success(), "git log thất bại: {}", out.stderr_lossy());

    let r = parse_log(&out.stdout);
    assert_eq!(
        r.skipped_records, 0,
        "repo mẫu không được có bản ghi méo (plan 02-02 đã bảo đảm)"
    );
    r.commits
}

/// Năm assertion **cứng**, chạy trên mọi fixture. Nằm ngoài snapshot có chủ ý — xem
/// ghi chú T-02-10 ở đầu tệp.
fn khang_dinh_cung(name: &str, commits: &[Commit], rows: &[GraphRow]) {
    // (1) Bất biến HIST-04 ở tầng dữ liệu: một hàng cho mỗi commit, đúng thứ tự.
    assert_eq!(
        rows.len(),
        commits.len(),
        "repo mẫu {name}: mất hoặc thừa hàng — {} commit vào, {} hàng ra",
        commits.len(),
        rows.len()
    );
    for (i, (c, r)) in commits.iter().zip(rows).enumerate() {
        assert_eq!(
            r.commit_id, c.id,
            "repo mẫu {name}: hàng {i} không khớp commit {i}"
        );
    }

    // (2) color == lane % LANE_COLORS ở mọi hàng — hợp đồng với bộ vẽ. Một chỉ số màu
    // tràn bảng cho `undefined` ở JavaScript và vẽ ra đường vô hình.
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(
            r.color,
            (r.lane % LANE_COLORS as u16) as u8,
            "repo mẫu {name} hàng {i}: màu {} không khớp lane {}",
            r.color,
            r.lane
        );
    }

    // (3) Cạnh **RẼ NHÁNH** (đi tới một lane khác lane của hàng) phải nằm trong giới
    // hạn hiển thị. Đó là điều `allocate_parent_lane` bảo đảm, và là chỗ neo của cách
    // vẽ suy giảm.
    //
    // # Vì sao không phải "mọi cạnh đều < MAX_VISIBLE_LANES" — đo được trên `wide`
    //
    // Bản đầu của test này khẳng định đúng câu đó và nó **đỏ thật trên fixture `wide`**
    // ở hàng 58 rồi hàng 60. Cả hai lần assertion là thứ sai, không phải `assign`:
    //
    //   r57 lane=0  trunc=1                     <-- lane 0..19 đã đầy, cắt cha thứ hai
    //   r58 lane=20 out=[(20,20)]               <-- đầu nhánh MỚI, lane 20 hợp lệ
    //   r60 lane=0  pass=[…, (20,20)]           <-- lane 20 đi xuyên qua
    //
    // `allocate_row_lane` **không** giới hạn, có chủ ý: r58 phải có hàng (HIST-04), và
    // lane thật của nó là 20. Một khi lane 20 sống, nó tất yếu đi xuyên qua các hàng
    // sau ở bước 4 — `pass=(20,20)` ở r60 là *hệ quả* của việc không bỏ hàng, không
    // phải một lỗi độc lập. Kẹp nó lại thì hoặc mất hàng, hoặc vẽ hai nhánh chồng lên
    // nhau ở cùng một lane.
    //
    // Hợp đồng đúng, ghi trong `graph/types.rs` và `docs/04-phase2-degraded-graph.md`:
    // Rust trả **lane thật**, frontend **gập** lane ≥ giới hạn vào cột cuối kèm chỉ
    // báo (plan 02-05). Điều `assign` bảo đảm là hẹp hơn nhưng đủ dùng: nó không bao
    // giờ **cấp mới** một lane rẽ nhánh vượt giới hạn — cha nào không còn lane thì vào
    // `truncated_parents`.
    for (i, r) in rows.iter().enumerate() {
        for e in &r.out_edges {
            if e.to_lane != r.lane {
                assert!(
                    e.to_lane < MAX_VISIBLE_LANES,
                    "repo mẫu {name} hàng {i}: cạnh RẼ NHÁNH tới lane {} vượt giới hạn \
                     {} — allocate_parent_lane đã hỏng, cha này phải vào \
                     truncated_parents thay vì được cấp lane",
                    e.to_lane,
                    MAX_VISIBLE_LANES
                );
            }
        }
    }

    // (4) Sổ sách cha phải cân ở mọi hàng: vẽ được + cắt bớt == số cha thật sự có
    // trong tập đã nạp. Không cha nào được biến mất khỏi sổ sách.
    let known: std::collections::HashSet<&str> = commits.iter().map(|c| c.id.as_str()).collect();
    for (i, (c, r)) in commits.iter().zip(rows).enumerate() {
        let cha_trong_tap = c
            .parents
            .iter()
            .filter(|p| known.contains(p.as_str()))
            .count();
        assert_eq!(
            r.out_edges.len() + r.truncated_parents as usize,
            cha_trong_tap,
            "repo mẫu {name} hàng {i}: {} cạnh + {} cắt != {} cha trong tập",
            r.out_edges.len(),
            r.truncated_parents,
            cha_trong_tap
        );
    }

    // (5) `terminates` chỉ được bật khi commit THẬT SỰ có cha ngoài tập đã nạp. Bật
    // sai là nói dối người dùng rằng lịch sử còn tiếp ở một chỗ nó đã hết.
    for (i, (c, r)) in commits.iter().zip(rows).enumerate() {
        let co_cha_ngoai_tap = c.parents.iter().any(|p| !known.contains(p.as_str()));
        assert_eq!(
            r.terminates, co_cha_ngoai_tap,
            "repo mẫu {name} hàng {i}: terminates={} nhưng cha ngoài tập={}",
            r.terminates, co_cha_ngoai_tap
        );
    }
}

/// Chạy một fixture: đọc git, gán lane, kiểm assertion cứng, trả về hàng để snapshot.
fn chay_fixture(name: &str) -> Option<(Vec<Commit>, Vec<GraphRow>)> {
    let repo = testing::require_fixture(name)?;
    let commits = doc_commits(&repo);
    assert!(!commits.is_empty(), "repo mẫu {name} phải có commit");
    let rows = assign(&commits);
    khang_dinh_cung(name, &commits, &rows);
    Some((commits, rows))
}

/// Sinh một test snapshot cho mỗi fixture.
macro_rules! test_fixture {
    ($ten_test:ident, $fixture:literal) => {
        #[test]
        fn $ten_test() {
            let Some((_, rows)) = chay_fixture($fixture) else {
                return; // fixture chưa sinh, đã in lời nhắc
            };
            insta::assert_snapshot!($fixture, chieu_gon(&rows));
        }
    };
}

test_fixture!(snapshot_linear, "linear");
test_fixture!(snapshot_octopus, "octopus");
test_fixture!(snapshot_unrelated, "unrelated");
test_fixture!(snapshot_orphan, "orphan");
test_fixture!(snapshot_wide, "wide");
test_fixture!(snapshot_non_utf8, "non-utf8");
test_fixture!(snapshot_shallow, "shallow");
test_fixture!(snapshot_detached, "detached");
test_fixture!(snapshot_submodule, "submodule");

// ---------------------------------------------------------------- assertion cứng riêng

/// Fixture `octopus`: tồn tại **đúng một** hàng có bốn cạnh ra.
///
/// Đây là ràng buộc số 3 của ROADMAP đo trên dữ liệu git thật, không trên buffer dựng
/// tay: một cài đặt chỉ đọc `parents[1]` cho hai cạnh và test này đỏ.
#[test]
fn octopus_co_dung_mot_hang_bon_canh_ra() {
    let Some((_, rows)) = chay_fixture("octopus") else {
        return;
    };

    let bon_canh: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.out_edges.len() == 4)
        .map(|(i, _)| i)
        .collect();

    assert_eq!(
        bon_canh.len(),
        1,
        "repo mẫu octopus phải có ĐÚNG MỘT merge bốn cha; thấy {} hàng: {:?}",
        bon_canh.len(),
        bon_canh
    );

    let m = &rows[bon_canh[0]];
    assert_eq!(
        m.truncated_parents, 0,
        "bốn lane thừa sức vẽ hết, không cắt"
    );

    // Bốn cạnh phải đi tới bốn lane KHÁC nhau — cùng một lane nghĩa là hai cha bị vẽ
    // chồng lên nhau và một nhánh biến mất khỏi hình.
    let mut dich: Vec<u16> = m.out_edges.iter().map(|e| e.to_lane).collect();
    dich.sort_unstable();
    let so_luong = dich.len();
    dich.dedup();
    assert_eq!(
        dich.len(),
        so_luong,
        "bốn cha của octopus phải đi tới bốn lane khác nhau"
    );
}

/// Fixture `shallow`: **git ghép biên nông**, nên `%P` của commit biên là RỖNG.
///
/// # Một điều plan và plan 02-02 đều đoán sai — đo được ở đây
///
/// Cả hai tài liệu viết rằng commit biên của bản sao nông *khai báo* một cha vắng mặt
/// (`7dae333f…`), và `assign` phải đặt `terminates` cho nó. Nửa đầu đúng với **đối
/// tượng commit trên đĩa**:
///
/// ```text
/// $ git cat-file -p 6b5521ba…
/// tree 82eb4fab…
/// parent 7dae333f…          <-- có
/// ```
///
/// Nhưng `parse_log` không đọc đối tượng commit, nó đọc `git log --format=%P`. Và git
/// **ghép** (graft) biên của bản sao nông: `%P` trả về rỗng.
///
/// ```text
/// $ git log --all --date-order --format='%H|%P'
/// 6b5521ba…|                 <-- rỗng, không phải 7dae333f…
/// ```
///
/// Nghĩa là: từ dữ liệu `assign` nhận được, biên nông **không phân biệt được** với gốc
/// thật, và `terminates == false` là câu trả lời **đúng** — `assign` không được phép
/// đoán ra một cha mà git đã cố tình che. Muốn vẽ dấu "còn tiếp" cho bản sao nông thì
/// phải đọc `.git/shallow` hoặc `git rev-parse --is-shallow-repository`, là việc của
/// tầng repository (plan 02-04), không phải của thuật toán lane.
///
/// Đường `terminates` vẫn cần thiết và vẫn được kiểm — nhưng ở ca **phân trang**, xem
/// [`phan_trang_sinh_terminates`]. Đó cũng là ca xảy ra thật trong sản phẩm, vì HIST-01
/// nạp lịch sử theo trang.
#[test]
fn shallow_git_ghep_bien_nen_khong_khai_bao_cha() {
    let Some((commits, rows)) = chay_fixture("shallow") else {
        return;
    };

    // Đúng một commit không có cha nào trong đầu ra của git log: biên nông đã bị ghép.
    let khong_cha: Vec<usize> = commits
        .iter()
        .enumerate()
        .filter(|(_, c)| c.parents.is_empty())
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        khong_cha.len(),
        1,
        "bản sao nông phải cho đúng một commit không cha (biên đã ghép), thấy {:?}",
        khong_cha
    );

    // Và vì git che cha đó, `terminates` phải là false ở mọi hàng — `assign` không
    // được bịa ra một cha mà dữ liệu không có.
    for (i, r) in rows.iter().enumerate() {
        assert!(
            !r.terminates,
            "hàng {i}: git đã ghép biên nông nên không cha nào ngoài tập; \
             terminates=true ở đây nghĩa là assign đang bịa dữ liệu"
        );
    }

    // Lane của biên nông phải được giải phóng, y như gốc thật.
    let bien = &rows[khong_cha[0]];
    assert!(
        bien.out_edges.is_empty(),
        "biên nông đã ghép không có cha nào để nối tới"
    );
}

/// **Ca `terminates` xảy ra thật trong sản phẩm: phân trang.**
///
/// HIST-01 nạp lịch sử theo trang (`--max-count`/`--skip`), nên hàng cuối của mỗi
/// trang gần như luôn khai báo một cha chưa nạp. Đó là lúc `terminates` phải bật để
/// frontend vẽ "còn tiếp" thay vì dấu chấm hết — vẽ nó như gốc là nói với người dùng
/// rằng lịch sử dừng ở giữa trang.
///
/// Đây là ca mà fixture `shallow` **được cho là** kiểm nhưng không kiểm được (xem
/// [`shallow_git_ghep_bien_nen_khong_khai_bao_cha`]), nên nó có test riêng ở đây và
/// dùng `--max-count` trên fixture `linear` — 20 commit một đường thẳng, nên cha vắng
/// mặt là do phân trang chứ không do hình dạng.
#[test]
fn phan_trang_sinh_terminates() {
    let Some(repo) = testing::require_fixture("linear") else {
        return;
    };

    let out = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("không dựng được tokio runtime")
        .block_on(async {
            GitCommand::new(&repo)
                .args(LOG_ARGS)
                .arg(LOG_FORMAT)
                .arg("--max-count=3")
                .run()
                .await
        })
        .expect("không chạy được git log");

    let commits = parse_log(&out.stdout).commits;
    assert_eq!(commits.len(), 3, "trang đầu phải có đúng ba commit");

    let rows = assign(&commits);
    khang_dinh_cung("linear (trang 3)", &commits, &rows);

    // Hai hàng đầu nối bình thường trong trang.
    assert!(!rows[0].terminates, "hàng 0 có cha nằm trong trang");
    assert!(!rows[1].terminates, "hàng 1 có cha nằm trong trang");

    // Hàng cuối khai báo một cha chưa nạp → terminates, lane kết thúc, không cạnh ra.
    assert!(
        !commits[2].parents.is_empty(),
        "hàng cuối trang phải KHAI BÁO cha (khác hẳn biên nông đã ghép)"
    );
    assert!(
        rows[2].terminates,
        "hàng cuối trang có cha chưa nạp → terminates phải true, \
         nếu không frontend sẽ vẽ nó như commit gốc"
    );
    assert!(
        rows[2].out_edges.is_empty(),
        "không vẽ cạnh tới một commit chưa nạp"
    );
}

/// Fixture `wide` — ba điều cùng lúc, và là chỗ HIST-04 gặp giới hạn hiển thị.
///
/// 1. Fixture thật sự rộng: lane lớn nhất ≥ 15.
/// 2. **Không hàng nào bị bỏ** dù fixture rộng hơn [`MAX_VISIBLE_LANES`] — đây là điều
///    quan trọng nhất. Một cài đặt áp giới hạn hiển thị vào lane của hàng sẽ đánh rơi
///    hàng ở đúng đây, trong khi mọi fixture nhỏ vẫn xanh.
/// 3. Lane của hàng **được phép** vượt giới hạn hiển thị; việc gập là của frontend.
///
/// In số thật ra stdout để chép vào SUMMARY và vào `docs/04-phase2-degraded-graph.md`.
#[test]
fn wide_rong_that_va_khong_bo_hang_nao() {
    let Some((commits, rows)) = chay_fixture("wide") else {
        return;
    };

    let max_lane = rows.iter().map(|r| r.lane).max().unwrap();
    let tong_cat: u32 = rows.iter().map(|r| r.truncated_parents as u32).sum();
    let so_hang_vuot = rows.iter().filter(|r| r.lane >= MAX_VISIBLE_LANES).count();
    let max_song = rows
        .iter()
        .map(|r| r.passthrough.len() + 1)
        .max()
        .unwrap_or(0);

    println!(
        "FIXTURE wide: {} commit, {} hàng, lane lớn nhất {}, \
         lane sống đồng thời lớn nhất {}, {} hàng vượt giới hạn {}, tổng cha bị cắt {}",
        commits.len(),
        rows.len(),
        max_lane,
        max_song,
        so_hang_vuot,
        MAX_VISIBLE_LANES,
        tong_cat
    );

    assert!(
        max_lane >= 15,
        "repo mẫu wide phải thật sự rộng (lane lớn nhất ≥ 15), thấy {max_lane}. \
         Số này nhỏ nghĩa là fixture đã hỏng, không phải assign đã đúng."
    );

    // Bất biến quan trọng nhất của test này.
    assert_eq!(
        rows.len(),
        commits.len(),
        "HIST-04 gặp giới hạn hiển thị: {} commit vào nhưng {} hàng ra. \
         Giới hạn hiển thị KHÔNG được làm mất hàng.",
        commits.len(),
        rows.len()
    );

    // Lane của hàng được phép vượt giới hạn — khẳng định tường minh để không ai "sửa"
    // thành kẹp lane vào giới hạn.
    assert!(
        max_lane >= MAX_VISIBLE_LANES,
        "repo mẫu wide ({max_lane} lane) phải vượt giới hạn {MAX_VISIBLE_LANES} — \
         nếu không thì nhánh vẽ suy giảm không được kiểm trên dữ liệu thật lần nào"
    );
}

/// Mọi fixture dựng được đều phải cho đồ thị hợp lệ — không hình dạng nào làm mất hàng.
///
/// Trùng một phần với các test riêng ở trên, có chủ ý: test này quét **cả chín** hình
/// dạng bằng cùng một bộ assertion, nên một fixture mới thêm vào `FIXTURE_NAMES` được
/// kiểm ngay mà không cần ai nhớ viết test cho nó.
#[test]
fn moi_fixture_cho_do_thi_hop_le() {
    let mut da_chay = 0;
    for name in testing::FIXTURE_NAMES {
        let Some(repo) = testing::fixture_path(name) else {
            continue; // test khác đã in lời nhắc
        };
        let commits = doc_commits(&repo);
        let rows = assign(&commits);
        khang_dinh_cung(name, &commits, &rows);

        let max_lane = rows.iter().map(|r| r.lane).max().unwrap_or(0);
        let tong_cat: u32 = rows.iter().map(|r| r.truncated_parents as u32).sum();
        println!(
            "{name}: {} commit, lane lớn nhất {}, cha bị cắt {}",
            commits.len(),
            max_lane,
            tong_cat
        );
        da_chay += 1;
    }

    assert!(
        da_chay > 0,
        "không fixture nào dựng được — chạy `bash scripts/fixtures/make-fixtures.sh`"
    );
}
