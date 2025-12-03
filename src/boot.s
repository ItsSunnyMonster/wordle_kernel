.global _start

.code32
_start:
    mov dword ptr [0xb8000], 0xd033d03a ; print :3 with a pink background
    hlt