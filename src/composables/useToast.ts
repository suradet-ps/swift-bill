import { ref, readonly } from "vue";

export type ToastType = "success" | "error" | "warning" | "info";

export interface Toast {
    id: number;
    type: ToastType;
    title: string;
    message?: string;
    duration: number;
    progress: number;
    paused: boolean;
}

const toasts = ref<Toast[]>([]);
let nextId = 0;

function addToast(
    type: ToastType,
    title: string,
    message?: string,
    duration: number = 4000
): number {
    const id = nextId++;
    const toast: Toast = { id, type, title, message, duration, progress: 100, paused: false };
    toasts.value.push(toast);

    // Auto-dismiss
    if (duration > 0) {
        const interval = 50;
        const decrement = (interval / duration) * 100;
        const timer = setInterval(() => {
            const t = toasts.value.find((t) => t.id === id);
            if (!t) {
                clearInterval(timer);
                return;
            }
            if (t.paused) return;
            t.progress -= decrement;
            if (t.progress <= 0) {
                clearInterval(timer);
                removeToast(id);
            }
        }, interval);
    }

    return id;
}

function removeToast(id: number) {
    const idx = toasts.value.findIndex((t) => t.id === id);
    if (idx !== -1) toasts.value.splice(idx, 1);
}

function pauseToast(id: number) {
    const t = toasts.value.find((t) => t.id === id);
    if (t) t.paused = true;
}

function resumeToast(id: number) {
    const t = toasts.value.find((t) => t.id === id);
    if (t) t.paused = false;
}

export function useToast() {
    return {
        toasts: readonly(toasts),

        success(title: string, message?: string, duration?: number) {
            return addToast("success", title, message, duration);
        },

        error(title: string, message?: string, duration?: number) {
            return addToast("error", title, message, duration ?? 6000);
        },

        warning(title: string, message?: string, duration?: number) {
            return addToast("warning", title, message, duration);
        },

        info(title: string, message?: string, duration?: number) {
            return addToast("info", title, message, duration ?? 3500);
        },

        remove: removeToast,
        pause: pauseToast,
        resume: resumeToast,
    };
}
