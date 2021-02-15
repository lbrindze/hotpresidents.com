
// js for hotpresidents 



var route = {};

console.log("this works");


document.addEventListener('DOMContentLoaded', function() { 
 var page = document.querySelector(".frame").getAttribute("data-page");
 if(page.length>0) route[page]();
});



route.score = function(){
    
    console.log("in function");
 let hotScore = Number(document.querySelector("._score_counter._hotus").getAttribute("data-count"));
 let notScore = Number(document.querySelector("._score_counter._notus").getAttribute("data-count"));

 
                        
 var calc = Math.max(hotScore,notScore);
 let hotOrCold = hotScore == calc && notScore != calc ? "._hotus" : notScore == calc && hotScore != calc ? "._notus" : false;
    
 if(hotOrCold){
   document.querySelector("._score_counter"+hotOrCold).classList.add("_active");       
  }
       
}// end of route.score






