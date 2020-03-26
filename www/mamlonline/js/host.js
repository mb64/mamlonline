$(()=>{
    $("#choose-answer").click(()=>{
        $("#choose-answer").addClass("chosen")
        $("#choose-chat").removeClass("chosen")
    })
    $("#choose-chat").click(()=>{
        $("#choose-chat").addClass("chosen")
        $("#choose-answer").removeClass("chosen")
    })
    $('#chat-input-school-select ').select2();
})
