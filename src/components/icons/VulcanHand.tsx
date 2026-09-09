/* Vulcan salute 🖖 — the Handy Flow brand mark as reusable inline SVG.
   Pure paths (no <text>/font dependency), matches the app icon artwork. */
const VulcanHand = ({
  width,
  height,
}: {
  width?: number | string;
  height?: number | string;
}) => (
  <svg
    width={width || 24}
    height={height || 24}
    viewBox="0 0 512 512"
    className="fill-text"
    xmlns="http://www.w3.org/2000/svg"
  >
    <g transform="translate(256 272) scale(9.6) translate(-24 -24)">
      <path d="M18.4 21.5 L10.8 8.6 Q9.9 7.1 8.5 8.0 Q7.2 8.9 8.0 10.4 L16.0 22.9 Z" />
      <path d="M19.4 22.3 L15.2 6.6 Q14.7 5.0 13.1 5.4 Q11.6 5.9 11.9 7.5 L16.9 23.4 Z" />
      <path d="M28.6 22.3 L32.4 6.5 Q32.9 4.9 34.5 5.3 Q36.0 5.8 35.7 7.4 L30.9 23.4 Z" />
      <path d="M29.6 21.5 L37.6 9.0 Q38.5 7.5 39.9 8.4 Q41.2 9.3 40.4 10.8 L32.2 23.0 Z" />
      <path d="M15.8 28.8 L5.2 23.6 Q3.7 22.8 2.9 24.2 Q2.2 25.6 3.7 26.5 L14.6 31.9 Z" />
      <path d="M16.8 20.0 Q24.0 17.6 31.2 20.0 L33.4 30.5 Q33.9 36.5 29.8 40.0 Q26.2 43.0 22.2 42.0 Q17.8 41.0 16.2 36.5 L14.5 30.5 Q14.0 27.0 16.8 20.0 Z" />
      <path d="M19.5 40.5 L28.5 40.5 L29.0 47.5 L19.0 47.5 Z" />
    </g>
  </svg>
);

export default VulcanHand;
