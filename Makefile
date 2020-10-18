MAKE = make -rm
NASM = nasm

OUTPUT_DIR := build
ASM_DIR := asm
OUTPUT_DIR_KEEP := $(OUTPUT_DIR)/.keep

$(OUTPUT_DIR)/ipl.bin: $(ASM_DIR)/ipl.asm Makefile $(OUTPUT_DIR_KEEP)
	$(NASM) $(ASM_DIR)/ipl.asm -o $(OUTPUT_DIR)/ipl.bin

$(OUTPUT_DIR)/asmhead.bin: $(ASM_DIR)/asmhead.asm Makefile $(OUTPUT_DIR_KEEP)
	$(NASM) $(ASM_DIR)/asnhead.asm -o $(OUTPUT_DIR)/asmhead.bin

default:
	$(MAKE) img

img:
	$(MAKE) $(OUTPUT_DIR)/asmhead.bin
	$(MAKE) $(OUTPUT_DIR)/ipl.bin
