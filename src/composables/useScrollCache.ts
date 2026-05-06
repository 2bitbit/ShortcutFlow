import { ref, onActivated, onDeactivated, nextTick, type Ref } from 'vue';

export function useScrollCache(scrollElementRef: Ref<HTMLElement | null | undefined>) {
  const scrollTop = ref(0);

  onDeactivated(() => {
    if (scrollElementRef.value) {
      scrollTop.value = scrollElementRef.value.scrollTop;
    }
  });

  onActivated(() => {
    if (scrollElementRef.value && scrollTop.value > 0) {
      nextTick(() => {
        if (scrollElementRef.value) {
          scrollElementRef.value.scrollTop = scrollTop.value;
        }
      });
    }
  });
}
