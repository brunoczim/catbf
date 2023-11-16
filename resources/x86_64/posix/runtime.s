	.file	"runtime.c"
	.text
	.section	.rodata
.LC0:
	.string	"stdio"
	.text
	.globl	main
	.type	main, @function
main:
.LFB6:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset 6, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register 6
	subq	$80, %rsp
	movl	%edi, -68(%rbp)
	movq	%rsi, -80(%rbp)
	movq	%fs:40, %rax
	movq	%rax, -8(%rbp)
	xorl	%eax, %eax
	movl	$0, -52(%rbp)
	movq	stdin(%rip), %rax
	movq	%rax, -32(%rbp)
	movq	stdout(%rip), %rax
	movq	%rax, -24(%rbp)
	movq	$8192, -48(%rbp)
	movq	-48(%rbp), %rax
	movq	%rax, %rdi
	call	malloc@PLT
	movq	%rax, -40(%rbp)
	movq	-48(%rbp), %rdx
	movq	-40(%rbp), %rcx
	leaq	-32(%rbp), %rax
	movq	%rcx, %rsi
	movq	%rax, %rdi
	call	fast_bfc_main@PLT
	movb	%al, -53(%rbp)
	cmpb	$0, -53(%rbp)
	jns	.L2
	movl	$1, -52(%rbp)
	leaq	.LC0(%rip), %rax
	movq	%rax, %rdi
	call	perror@PLT
.L2:
	movq	-40(%rbp), %rax
	movq	%rax, %rdi
	call	free@PLT
	movl	-52(%rbp), %eax
	movq	-8(%rbp), %rdx
	subq	%fs:40, %rdx
	je	.L4
	call	__stack_chk_fail@PLT
.L4:
	leave
	.cfi_def_cfa 7, 8
	ret
	.cfi_endproc
.LFE6:
	.size	main, .-main
	.globl	fast_bfc_put
	.type	fast_bfc_put, @function
fast_bfc_put:
.LFB7:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset 6, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register 6
	subq	$16, %rsp
	movq	%rdi, -8(%rbp)
	movl	%esi, %eax
	movb	%al, -12(%rbp)
	movq	-8(%rbp), %rax
	movq	8(%rax), %rdx
	movzbl	-12(%rbp), %eax
	movq	%rdx, %rsi
	movl	%eax, %edi
	call	fputc@PLT
	leave
	.cfi_def_cfa 7, 8
	ret
	.cfi_endproc
.LFE7:
	.size	fast_bfc_put, .-fast_bfc_put
	.globl	fast_bfc_get
	.type	fast_bfc_get, @function
fast_bfc_get:
.LFB8:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset 6, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register 6
	subq	$32, %rsp
	movq	%rdi, -24(%rbp)
	movq	-24(%rbp), %rax
	movq	(%rax), %rax
	movq	%rax, %rdi
	call	fgetc@PLT
	movw	%ax, -2(%rbp)
	movq	-24(%rbp), %rax
	movq	(%rax), %rax
	movq	%rax, %rdi
	call	feof@PLT
	testl	%eax, %eax
	je	.L8
	movl	$0, %eax
	jmp	.L9
.L8:
	cmpw	$0, -2(%rbp)
	jns	.L10
	movl	$-1, %eax
	jmp	.L9
.L10:
	movzwl	-2(%rbp), %eax
	sall	$8, %eax
	orl	$1, %eax
.L9:
	leave
	.cfi_def_cfa 7, 8
	ret
	.cfi_endproc
.LFE8:
	.size	fast_bfc_get, .-fast_bfc_get
	.globl	fast_bfc_tape_grow_next
	.type	fast_bfc_tape_grow_next, @function
fast_bfc_tape_grow_next:
.LFB9:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset 6, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register 6
	subq	$16, %rsp
	movq	%rdi, -8(%rbp)
	movq	%rsi, -16(%rbp)
	movq	-16(%rbp), %rax
	leaq	8192(%rax), %rdx
	movq	-8(%rbp), %rax
	movq	%rdx, %rsi
	movq	%rax, %rdi
	call	realloc@PLT
	leave
	.cfi_def_cfa 7, 8
	ret
	.cfi_endproc
.LFE9:
	.size	fast_bfc_tape_grow_next, .-fast_bfc_tape_grow_next
	.globl	fast_bfc_tape_grow_prev
	.type	fast_bfc_tape_grow_prev, @function
fast_bfc_tape_grow_prev:
.LFB10:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset 6, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register 6
	subq	$32, %rsp
	movq	%rdi, -24(%rbp)
	movq	%rsi, -32(%rbp)
	movq	-32(%rbp), %rax
	leaq	8192(%rax), %rdx
	movq	-24(%rbp), %rax
	movq	%rdx, %rsi
	movq	%rax, %rdi
	call	realloc@PLT
	movq	%rax, -8(%rbp)
	movq	-8(%rbp), %rax
	leaq	8192(%rax), %rcx
	movq	-8(%rbp), %rax
	movl	$8192, %edx
	movq	%rax, %rsi
	movq	%rcx, %rdi
	call	memmove@PLT
	movq	-8(%rbp), %rax
	leave
	.cfi_def_cfa 7, 8
	ret
	.cfi_endproc
.LFE10:
	.size	fast_bfc_tape_grow_prev, .-fast_bfc_tape_grow_prev
	.ident	"GCC: (GNU) 13.2.1 20230801"
	.section	.note.GNU-stack,"",@progbits
