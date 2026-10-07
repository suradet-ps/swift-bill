export const THAI_MONTHS = [
    "มกราคม", "กุมภาพันธ์", "มีนาคม", "เมษายน", "พฤษภาคม", "มิถุนายน",
    "กรกฎาคม", "สิงหาคม", "กันยายน", "ตุลาคม", "พฤศจิกายน", "ธันวาคม",
];

export const THAI_MONTHS_SHORT = [
    "ม.ค.", "ก.พ.", "มี.ค.", "เม.ย.", "พ.ค.", "มิ.ย.",
    "ก.ค.", "ส.ค.", "ก.ย.", "ต.ค.", "พ.ย.", "ธ.ค.",
];

export function formatMoney(n: number): string {
    return n.toLocaleString("th-TH", {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
    });
}

export function fileName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
}

/** Buddhist-era date from a YYYYMMDD API string, e.g. "1 ตุลาคม 2568". */
export function formatBuddhistDate(yyyymmdd: string): string {
    if (!yyyymmdd || yyyymmdd.length < 8) return yyyymmdd;
    const y = parseInt(yyyymmdd.substring(0, 4), 10) + 543;
    const m = parseInt(yyyymmdd.substring(4, 6), 10);
    const d = parseInt(yyyymmdd.substring(6, 8), 10);
    return `${d} ${THAI_MONTHS[m - 1] ?? ""} ${y}`;
}

/** Thai short date from a native date input value, e.g. "1 ต.ค. 68". */
export function toThaiShortDate(htmlDate: string): string {
    if (!htmlDate) return "";
    const parts = htmlDate.split("-");
    if (parts.length !== 3) return "";
    const year = parseInt(parts[0], 10);
    const month = parseInt(parts[1], 10);
    const day = parseInt(parts[2], 10);
    if (isNaN(year) || isNaN(month) || isNaN(day)) return "";
    const thaiYear = (year + 543) % 100;
    return `${day} ${THAI_MONTHS_SHORT[month - 1] ?? ""} ${thaiYear}`;
}

/** Thai full date from a native date input value, e.g. "1 ตุลาคม 2568". */
export function toThaiFullDate(htmlDate: string): string {
    if (!htmlDate) return "";
    const parts = htmlDate.split("-");
    if (parts.length !== 3) return "";
    const year = parseInt(parts[0], 10);
    const month = parseInt(parts[1], 10);
    const day = parseInt(parts[2], 10);
    if (isNaN(year) || isNaN(month) || isNaN(day)) return "";
    return `${day} ${THAI_MONTHS[month - 1] ?? ""} ${year + 543}`;
}

export function formatDateTime(iso: string): string {
    try {
        return new Date(iso).toLocaleString("th-TH", {
            year: "numeric",
            month: "short",
            day: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    } catch {
        return iso;
    }
}

/** Strip leading icons from legacy stored labels, e.g. an icon prefix before "เบิกยาปะหน้า". */
export function cleanSourceLabel(value: string): string {
    return value.replace(/^[^0-9A-Za-zก-๙]+/u, "").trim();
}

/** Period label from a YYYY-MM-DD range, e.g. "1-10 ตุลาคม 2568". */
export function formatPeriodLabel(startHtml: string, endHtml: string): string {
    if (!startHtml || !endHtml) return "";
    const sy = parseInt(startHtml.substring(0, 4), 10) + 543;
    const sm = parseInt(startHtml.substring(5, 7), 10);
    const sd = parseInt(startHtml.substring(8, 10), 10);
    const ey = parseInt(endHtml.substring(0, 4), 10) + 543;
    const em = parseInt(endHtml.substring(5, 7), 10);
    const ed = parseInt(endHtml.substring(8, 10), 10);
    if (sm === em && sy === ey) {
        return `${sd}-${ed} ${THAI_MONTHS[sm - 1]} ${sy}`;
    }
    return `${sd} ${THAI_MONTHS_SHORT[sm - 1]} - ${ed} ${THAI_MONTHS_SHORT[em - 1]} ${ey}`;
}
