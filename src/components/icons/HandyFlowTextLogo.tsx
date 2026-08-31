/* eslint-disable i18next/no-literal-string */
const HandyFlowTextLogo = ({
  width = 120,
  className = "",
}: {
  width?: number | string;
  className?: string;
}) => (
  <svg
    width={width}
    viewBox="0 0 460 84"
    className={className}
    role="img"
    aria-label="Handy Flow"
  >
    <defs>
      <linearGradient id="hf-wave" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" stopColor="#38bdf8" />
        <stop offset="1" stopColor="#a78bfa" />
      </linearGradient>
    </defs>
    <text
      x="0"
      y="62"
      fontFamily="Segoe UI, Arial, sans-serif"
      fontSize="58"
      fontWeight="700"
      fill="url(#hf-wave)"
    >
      Handy Flow
    </text>
  </svg>
);

export default HandyFlowTextLogo;
