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
    viewBox="0 0 400 84"
    className={className}
    role="img"
    aria-label="Handy Flow"
    style={{ maxWidth: "100%", height: "auto" }}
  >
    <defs>
      <linearGradient id="hf-wave" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" stopColor="var(--hf-primary-500)" />
        <stop offset="1" stopColor="var(--hf-primary-400)" />
      </linearGradient>
    </defs>
    <text
      x="0"
      y="60"
      fontFamily="Segoe UI, Arial, sans-serif"
      fontSize="48"
      fontWeight="700"
      letterSpacing="0.5"
      fill="url(#hf-wave)"
    >
      HandyFlow
    </text>
  </svg>
);

export default HandyFlowTextLogo;
