<template>
  <svg
    class="brand-wave"
    :class="{ active }"
    viewBox="0 0 1000 320"
    preserveAspectRatio="none"
    aria-hidden="true"
  >
    <g
      v-for="layer in layers"
      :key="layer.index"
      class="wave-layer"
      :style="{
        '--wave-shift': `${-layer.length}px`,
        '--wave-duration': `${layer.duration}s`,
      }"
    >
      <path :d="layer.fill" fill="currentColor" :fill-opacity="layer.fillOpacity" />
      <path
        v-if="layer.strokeWidth > 0"
        :d="layer.crest"
        fill="none"
        stroke="currentColor"
        :stroke-opacity="layer.strokeOpacity"
        :stroke-width="layer.strokeWidth"
        stroke-linecap="round"
      />
    </g>
  </svg>
</template>

<script lang="ts" setup>
/**
 * Знак бренда: три слоя настоящего синуса. Формула повторяет телефон
 * (mobile/app/.../ui/HomeHero.kt → drawBrandWave), поэтому волна на всех
 * носителях одна и та же: у каждого слоя своя длина и своя скорость, гребни
 * расходятся — вода живёт, а не едет картинкой.
 *
 * Рисуем разметкой, а не холстом: цвет берётся темой (`currentColor`), и окно,
 * открытое часами, не крутит из-за фона ни одного лишнего кадра.
 */

defineOptions({ name: 'BrandWave' })

withDefaults(defineProps<{ active?: boolean }>(), { active: false })

const WIDTH = 1000
const HEIGHT = 320
const STEP = WIDTH / 96

const LAYERS = [
  { base: 0.63, amp: 0.082, length: 1.02, duration: 14, strokeWidth: 2.5 },
  { base: 0.7, amp: 0.06, length: 0.78, duration: 18, strokeWidth: 1.5 },
  { base: 0.77, amp: 0.044, length: 0.58, duration: 25, strokeWidth: 0 },
]

const buildLayer = (
  layer: (typeof LAYERS)[number],
  index: number,
) => {
  const baseY = HEIGHT * layer.base
  const amp = WIDTH * layer.amp
  const length = WIDTH * layer.length
  const shift = index * 0.9
  // Путь длиннее окна ровно на одну волну: сдвиг на неё возвращает картинку
  // в исходное положение, поэтому движение бесшовно.
  const till = WIDTH + length
  const points: string[] = []
  for (let x = 0; x <= till; x += STEP) {
    const y = baseY + amp * Math.sin((2 * Math.PI * x) / length + shift)
    points.push(`${x.toFixed(1)},${y.toFixed(1)}`)
  }
  const crest = `M${points.join(' L')}`
  return {
    index,
    length,
    duration: layer.duration,
    crest,
    fill: `${crest} L${till.toFixed(1)},${HEIGHT} L0,${HEIGHT} Z`,
    fillOpacity: (0.1 - index * 0.028).toFixed(3),
    strokeOpacity: (0.3 - index * 0.14).toFixed(2),
    strokeWidth: layer.strokeWidth,
  }
}

const layers = LAYERS.map(buildLayer)
</script>

<style scoped>
.brand-wave {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  height: clamp(180px, 34vh, 320px);
  pointer-events: none;
  color: var(--wave-color, var(--primary-color));
  transition: color var(--transition-slow);
}

.wave-layer {
  transform: translateX(0);
}

.brand-wave.active .wave-layer {
  animation: wave-drift var(--wave-duration) linear infinite;
}

@keyframes wave-drift {
  to {
    transform: translateX(var(--wave-shift));
  }
}

@media (prefers-reduced-motion: reduce) {
  .brand-wave.active .wave-layer {
    animation: none;
  }
}
</style>
