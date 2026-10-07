<script setup lang="ts">
import { useToast } from "../composables/useToast";

const { toasts, remove, pause, resume } = useToast();

const icons: Record<string, string> = {
    success: `<svg width="20" height="20" viewBox="0 0 20 20" fill="none"><circle cx="10" cy="10" r="10" fill="currentColor" opacity="0.15"/><path d="M6 10.5l2.5 2.5L14 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
    error: `<svg width="20" height="20" viewBox="0 0 20 20" fill="none"><circle cx="10" cy="10" r="10" fill="currentColor" opacity="0.15"/><path d="M7 7l6 6M13 7l-6 6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>`,
    warning: `<svg width="20" height="20" viewBox="0 0 20 20" fill="none"><circle cx="10" cy="10" r="10" fill="currentColor" opacity="0.15"/><path d="M10 6v5" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><circle cx="10" cy="14" r="1.2" fill="currentColor"/></svg>`,
    info: `<svg width="20" height="20" viewBox="0 0 20 20" fill="none"><circle cx="10" cy="10" r="10" fill="currentColor" opacity="0.15"/><circle cx="10" cy="6.5" r="1.2" fill="currentColor"/><path d="M10 9v5" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>`,
};
</script>

<template>
<Teleport to="body">
    <div class="toast-container" aria-live="polite">
        <TransitionGroup name="toast">
            <div
                v-for="toast in toasts"
                :key="toast.id"
                :class="['toast', `toast--${toast.type}`]"
                @mouseenter="pause(toast.id)"
                @mouseleave="resume(toast.id)"
                role="alert"
            >
                <div class="toast__icon" v-html="icons[toast.type]"></div>
                <div class="toast__body">
                    <div class="toast__title">{{ toast.title }}</div>
                    <div v-if="toast.message" class="toast__message">{{ toast.message }}</div>
                </div>
                <button class="toast__close" @click="remove(toast.id)" aria-label="ปิด">
                    <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                        <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
                    </svg>
                </button>
                <div class="toast__progress">
                    <div
                        class="toast__progress-bar"
                        :style="{ width: toast.progress + '%' }"
                    ></div>
                </div>
            </div>
        </TransitionGroup>
    </div>
</Teleport>
</template>

<style>
/* Toast container */
.toast-container {
    position: fixed;
    top: 16px;
    right: 16px;
    z-index: 99999;
    display: flex;
    flex-direction: column;
    gap: 10px;
    pointer-events: none;
    max-width: 420px;
    width: calc(100% - 32px);
}

/* Individual toast */
.toast {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 14px 15px 16px;
    border-radius: var(--radius-lg);
    background: var(--c-surface);
    border: none;
    box-shadow: var(--shadow-pop);
    position: relative;
    overflow: hidden;
    cursor: default;
    min-width: 320px;
}

/* Type variants - colors come from theme tokens, dark mode included */
.toast--success {
    color: var(--c-success);
}
.toast--success .toast__progress-bar {
    background: var(--c-success);
}

.toast--error {
    color: var(--c-error);
}
.toast--error .toast__progress-bar {
    background: var(--c-error);
}

.toast--warning {
    color: var(--c-warn);
}
.toast--warning .toast__progress-bar {
    background: #d97706;
}

.toast--info {
    color: var(--c-primary);
}
.toast--info .toast__progress-bar {
    background: var(--c-primary);
}

.toast__icon {
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    margin-top: 1px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.toast__body {
    flex: 1;
    min-width: 0;
}

.toast__title {
    font-size: var(--fs-body);
    font-weight: 600;
    line-height: 1.35;
    letter-spacing: 0;
    color: inherit;
}

.toast__message {
    font-size: var(--fs-sm);
    line-height: 1.55;
    margin-top: 3px;
    opacity: 0.85;
    color: var(--c-text-muted);
    word-break: break-word;
}

.toast__close {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--c-text-light);
    cursor: pointer;
    transition: background var(--dur-1), color var(--dur-1);
    margin: -2px -4px 0 0;
}

.toast__close:hover {
    background: var(--c-primary-light);
    color: var(--c-primary);
}

.toast__progress {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: var(--c-border-soft);
}

.toast__progress-bar {
    height: 100%;
    border-radius: 0 0 3px 3px;
    transition: width 0.05s linear;
}

/* Transitions */
.toast-enter-active {
    animation: toast-in 0.35s cubic-bezier(0.22, 1, 0.36, 1);
}

.toast-leave-active {
    animation: toast-out 0.28s cubic-bezier(0.55, 0, 1, 0.45) forwards;
}

.toast-move {
    transition: transform 0.3s cubic-bezier(0.22, 1, 0.36, 1);
}

@keyframes toast-in {
    0% {
        opacity: 0;
        transform: translateX(80px) scale(0.92);
    }
    100% {
        opacity: 1;
        transform: translateX(0) scale(1);
    }
}

@keyframes toast-out {
    0% {
        opacity: 1;
        transform: translateX(0) scale(1);
    }
    100% {
        opacity: 0;
        transform: translateX(80px) scale(0.92);
    }
}
</style>
